use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::{
    app::{security::enforce_rate_limit, state::AppState},
    features::auth::models::{AuthResponse, BnetCompleteInput},
    shared::{errors::ApiResult, models::MessageResponse},
};

use super::{battlenet_service, service};
use super::battlenet_service::BnetCallbackResult;

fn nonce_cookie(nonce: &str, redirect_uri: &str, max_age: u32) -> String {
    let secure = if redirect_uri.starts_with("https") { "; Secure" } else { "" };
    format!(
        "bnet_nonce={}; HttpOnly; SameSite=Lax{}; Path=/api/auth/battlenet/callback; Max-Age={}",
        nonce, secure, max_age
    )
}

pub async fn battlenet_authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    info!("battlenet authorize started");

    if enforce_rate_limit(&state.rate_limiter, &headers, "bnet_authorize", 20, 60)
        .await
        .is_err()
    {
        warn!("battlenet authorize rate limited");
        return Redirect::to(&format!(
            "{}/auth/callback?error=rate_limited",
            state.config.frontend_url,
        ))
        .into_response();
    }

    let oauth_not_configured = state.config.battlenet_client_id.trim().is_empty()
        || state.config.battlenet_client_secret.trim().is_empty()
        || state.config.battlenet_redirect_uri.trim().is_empty();

    if oauth_not_configured {
        error!("battlenet authorize failed: oauth_not_configured");
        return Redirect::to(&format!(
            "{}/auth/callback?error=oauth_not_configured",
            state.config.frontend_url
        ))
        .into_response();
    }

    let (csrf_state, nonce) = match battlenet_service::build_oauth_state(&state.config.jwt_secret) {
        Ok(result) => result,
        Err(_) => {
            error!("battlenet authorize failed: oauth_state_generation_failed");
            return Redirect::to(&format!(
                "{}/auth/callback?error=oauth_state_generation_failed",
                state.config.frontend_url,
            ))
            .into_response();
        }
    };
    let url = format!(
        "https://oauth.battle.net/authorize?client_id={}&scope=openid&state={}&redirect_uri={}&response_type=code",
        urlencoding::encode(&state.config.battlenet_client_id),
        urlencoding::encode(&csrf_state),
        urlencoding::encode(&state.config.battlenet_redirect_uri),
    );
    info!("battlenet authorize redirect generated");
    let cookie = nonce_cookie(&nonce, &state.config.battlenet_redirect_uri, 600);
    let mut response = Redirect::to(&url).into_response();
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, val);
    }
    response
}

#[derive(Deserialize)]
pub struct BnetCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub async fn battlenet_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<BnetCallbackParams>,
) -> Response {
    let nonce = extract_session_nonce(&headers);
    let clear_nonce = nonce_cookie("", &state.config.battlenet_redirect_uri, 0);
    let redir = battlenet_callback_inner(&state, &headers, params, nonce.as_deref()).await;
    let mut response = redir.into_response();
    if let Ok(val) = HeaderValue::from_str(&clear_nonce) {
        response.headers_mut().insert(header::SET_COOKIE, val);
    }
    response
}

async fn battlenet_callback_inner(
    state: &AppState,
    headers: &HeaderMap,
    params: BnetCallbackParams,
    nonce: Option<&str>,
) -> Redirect {
    info!("battlenet callback received");
    let frontend_url = &state.config.frontend_url;

    if enforce_rate_limit(&state.rate_limiter, headers, "bnet_callback", 20, 60)
        .await
        .is_err()
    {
        warn!("battlenet callback rate limited");
        return Redirect::to(&format!("{frontend_url}/auth/callback?error=rate_limited"));
    }

    if let Some(error) = &params.error {
        warn!(oauth_error = %error, "battlenet callback returned provider error");
        return Redirect::to(&format!(
            "{}/auth/callback?error={}",
            frontend_url,
            urlencoding::encode(error),
        ));
    }

    let (Some(code), Some(csrf_state)) =
        (params.code.as_deref(), params.state.as_deref())
    else {
        warn!("battlenet callback missing code or state query parameters");
        return Redirect::to(&format!("{frontend_url}/auth/callback?error=missing_params"));
    };

    let nonce = match nonce {
        Some(n) => n,
        None => {
            warn!("battlenet callback missing nonce cookie; possible CSRF attempt");
            return Redirect::to(&format!("{frontend_url}/auth/callback?error=invalid_state"));
        }
    };

    match battlenet_service::handle_battlenet_redirect(state, code, csrf_state, nonce).await {
        Ok(BnetCallbackResult::LoggedIn(auth)) => {
            info!("battlenet callback completed: login flow succeeded");
            Redirect::to(&format!(
                "{}/auth/callback#access_token={}&refresh_token={}",
                frontend_url,
                urlencoding::encode(&auth.access_token),
                urlencoding::encode(&auth.refresh_token),
            ))
        }
        Ok(BnetCallbackResult::Connected(user_id)) => {
            info!(%user_id, "battlenet callback completed: connect flow succeeded");
            Redirect::to(&format!(
                "{}/auth/callback?connected=true&profile_id={}",
                frontend_url,
                urlencoding::encode(&user_id.to_string()),
            ))
        }
        Ok(BnetCallbackResult::RequiresEmail { pending_token, battletag }) => {
            info!("battlenet callback requires email completion");
            Redirect::to(&format!(
                "{}/auth/callback#needs_email=true&pending_token={}&battletag={}",
                frontend_url,
                urlencoding::encode(&pending_token),
                urlencoding::encode(&battletag),
            ))
        }
        Err(_err) => {
            error!("battlenet callback failed");
            Redirect::to(&format!("{frontend_url}/auth/callback?error=authentication_failed"))
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
            (k.trim() == "bnet_nonce").then(|| v.trim().to_string())
        })
}

pub async fn battlenet_complete_signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<BnetCompleteInput>,
) -> ApiResult<AuthResponse> {
    enforce_rate_limit(&state.rate_limiter, &headers, "bnet_complete", 20, 60).await?;
    battlenet_service::complete_battlenet_signup(&state, &payload.pending_token, &payload.email)
        .await
        .map(Json)
}

#[derive(Deserialize)]
pub struct ConnectInitParams {
    pub token: Option<String>,
}

pub async fn battlenet_connect_init(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ConnectInitParams>,
) -> Response {
    let frontend_url = &state.config.frontend_url;

    if enforce_rate_limit(&state.rate_limiter, &headers, "bnet_connect_init", 20, 60)
        .await
        .is_err()
    {
        warn!("battlenet connect-init rate limited");
        return Redirect::to(&format!(
            "{}/auth/callback?error=rate_limited",
            frontend_url,
        ))
        .into_response();
    }

    let token = match params.token.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(t) => t,
        None => {
            return Redirect::to(&format!(
                "{}/auth/callback?error=unauthorized",
                frontend_url,
            ))
            .into_response();
        }
    };

    let user_id = match service::verify_access_token_str(&state, token) {
        Ok(id) => id,
        Err(_) => {
            return Redirect::to(&format!(
                "{}/auth/callback?error=unauthorized",
                frontend_url,
            ))
            .into_response();
        }
    };

    let (url, nonce) = match battlenet_service::battlenet_connect_init_url(&state, user_id).await {
        Ok(result) => result,
        Err(_) => {
            return Redirect::to(&format!(
                "{}/auth/callback?error=connect_init_failed",
                frontend_url,
            ))
            .into_response();
        }
    };

    let cookie = nonce_cookie(&nonce, &state.config.battlenet_redirect_uri, 600);
    let mut response = Redirect::to(&url).into_response();
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, val);
    }
    response
}

pub async fn battlenet_disconnect(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    battlenet_service::disconnect_battlenet(&state, user_id).await?;
    Ok(Json(MessageResponse {
        message: "Battle.net account disconnected".to_string(),
    }))
}
