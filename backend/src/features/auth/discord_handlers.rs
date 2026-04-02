use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    app::{security::enforce_rate_limit, state::AppState},
    shared::{
        errors::{internal_error, ApiError, ApiResult},
        models::MessageResponse,
    },
};

use super::{discord_service, service};
use super::discord_service::DiscordCallbackResult;

fn nonce_cookie(nonce: &str, redirect_uri: &str, max_age: u32) -> String {
    let secure = if redirect_uri.starts_with("https") { "; Secure" } else { "" };
    format!(
        "discord_nonce={}; HttpOnly; SameSite=Lax{}; Path=/api/auth/discord/callback; Max-Age={}",
        nonce, secure, max_age
    )
}

pub async fn discord_authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    info!("discord authorize started");

    if enforce_rate_limit(&state.rate_limiter, &headers, "discord_authorize", 20, 60)
        .await
        .is_err()
    {
        warn!("discord authorize rate limited");
        return Redirect::to(&format!(
            "{}/auth/callback?error=rate_limited",
            state.config.frontend_url,
        ))
        .into_response();
    }

    let oauth_not_configured = state.config.discord_client_id.trim().is_empty()
        || state.config.discord_client_secret.trim().is_empty()
        || state.config.discord_redirect_uri.trim().is_empty();

    if oauth_not_configured {
        error!("discord authorize failed: oauth_not_configured");
        return Redirect::to(&format!(
            "{}/auth/callback?error=oauth_not_configured",
            state.config.frontend_url
        ))
        .into_response();
    }

    let (csrf_state, nonce) =
        match discord_service::build_discord_oauth_state(&state.config.jwt_secret) {
            Ok(result) => result,
            Err(_) => {
                error!("discord authorize failed: oauth_state_generation_failed");
                return Redirect::to(&format!(
                    "{}/auth/callback?error=oauth_state_generation_failed",
                    state.config.frontend_url,
                ))
                .into_response();
            }
        };

    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&scope=identify+email&state={}&redirect_uri={}&response_type=code",
        urlencoding::encode(&state.config.discord_client_id),
        urlencoding::encode(&csrf_state),
        urlencoding::encode(&state.config.discord_redirect_uri),
    );

    info!("discord authorize redirect generated");
    let cookie = nonce_cookie(&nonce, &state.config.discord_redirect_uri, 600);
    let mut response = Redirect::to(&url).into_response();
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, val);
    }
    response
}

#[derive(Deserialize)]
pub struct DiscordCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub async fn discord_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<DiscordCallbackParams>,
) -> Response {
    let nonce = extract_session_nonce(&headers);
    let clear_nonce = nonce_cookie("", &state.config.discord_redirect_uri, 0);
    let redir = discord_callback_inner(&state, &headers, params, nonce.as_deref()).await;
    let mut response = redir.into_response();
    if let Ok(val) = HeaderValue::from_str(&clear_nonce) {
        response.headers_mut().insert(header::SET_COOKIE, val);
    }
    response
}

async fn discord_callback_inner(
    state: &AppState,
    headers: &HeaderMap,
    params: DiscordCallbackParams,
    nonce: Option<&str>,
) -> Redirect {
    info!("discord callback received");
    let frontend_url = &state.config.frontend_url;

    if enforce_rate_limit(&state.rate_limiter, headers, "discord_callback", 20, 60)
        .await
        .is_err()
    {
        warn!("discord callback rate limited");
        return Redirect::to(&format!("{frontend_url}/auth/callback?error=rate_limited"));
    }

    if let Some(error) = &params.error {
        warn!(oauth_error = %error, "discord callback returned provider error");
        return Redirect::to(&format!(
            "{}/auth/callback?error={}",
            frontend_url,
            urlencoding::encode(error),
        ));
    }

    let (Some(code), Some(csrf_state)) = (params.code.as_deref(), params.state.as_deref()) else {
        warn!("discord callback missing code or state query parameters");
        return Redirect::to(&format!("{frontend_url}/auth/callback?error=missing_params"));
    };

    let nonce = match nonce {
        Some(n) => n,
        None => {
            warn!("discord callback missing nonce cookie; possible CSRF attempt");
            return Redirect::to(&format!("{frontend_url}/auth/callback?error=invalid_state"));
        }
    };

    match discord_service::handle_discord_redirect(state, code, csrf_state, nonce).await {
        Ok(DiscordCallbackResult::LoggedIn(auth)) => {
            info!("discord callback completed: login flow succeeded");
            Redirect::to(&format!(
                "{}/auth/callback#access_token={}&refresh_token={}",
                frontend_url,
                urlencoding::encode(&auth.access_token),
                urlencoding::encode(&auth.refresh_token),
            ))
        }
        Ok(DiscordCallbackResult::Connected(user_id)) => {
            info!(%user_id, "discord callback completed: connect flow succeeded");
            Redirect::to(&format!(
                "{}/auth/callback?connected=true&profile_id={}",
                frontend_url,
                urlencoding::encode(&user_id.to_string()),
            ))
        }
        Err(err) => {
            let (_, axum::Json(body)) = err;
            error!(error = %body.error, "discord callback failed");
            Redirect::to(&format!(
                "{frontend_url}/auth/callback?error=authentication_failed"
            ))
        }
    }
}

fn extract_session_nonce(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k.trim() == "discord_nonce").then(|| v.trim().to_string())
        })
}

pub async fn discord_connect_init(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    enforce_rate_limit(&state.rate_limiter, &headers, "discord_connect_init", 20, 60).await?;
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    let (url, nonce) = discord_service::discord_connect_init_url(&state, user_id).await?;
    let cookie = nonce_cookie(&nonce, &state.config.discord_redirect_uri, 600);
    let cookie_val = HeaderValue::from_str(&cookie).map_err(internal_error)?;
    #[derive(Serialize)]
    struct ConnectInitResponse { redirect_url: String }
    Ok((
        [(header::SET_COOKIE, cookie_val)],
        Json(ConnectInitResponse { redirect_url: url }),
    ))
}

pub async fn discord_disconnect(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    discord_service::disconnect_discord(&state, user_id).await?;
    Ok(axum::Json(MessageResponse {
        message: "Discord account disconnected".to_string(),
    }))
}
