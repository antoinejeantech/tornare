use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Redirect,
    Json,
};
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::{
    app::{security::enforce_rate_limit, state::AppState},
    features::auth::models::{AuthResponse, BnetCompleteInput, BnetConnectInitResponse},
    shared::{errors::ApiResult, models::MessageResponse},
};

use super::{battlenet_service, service};
use super::battlenet_service::BnetCallbackResult;

pub async fn battlenet_authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Redirect {
    info!("battlenet authorize started");

    if enforce_rate_limit(&state.rate_limiter, &headers, "bnet_authorize", 20, 60)
        .await
        .is_err()
    {
        warn!("battlenet authorize rate limited");
        return Redirect::to(&format!(
            "{}/auth/callback?error=rate_limited",
            state.config.frontend_url,
        ));
    }

    let oauth_not_configured = state.config.battlenet_client_id.trim().is_empty()
        || state.config.battlenet_client_secret.trim().is_empty()
        || state.config.battlenet_redirect_uri.trim().is_empty();

    if oauth_not_configured {
        error!("battlenet authorize failed: oauth_not_configured");
        return Redirect::to(&format!(
            "{}/auth/callback?error=oauth_not_configured",
            state.config.frontend_url
        ));
    }

    let csrf_state = battlenet_service::build_oauth_state(&state.config.jwt_secret);
    let url = format!(
        "https://oauth.battle.net/authorize?client_id={}&scope=openid&state={}&redirect_uri={}&response_type=code",
        urlencoding::encode(&state.config.battlenet_client_id),
        urlencoding::encode(&csrf_state),
        urlencoding::encode(&state.config.battlenet_redirect_uri),
    );
    info!("battlenet authorize redirect generated");
    Redirect::to(&url)
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
) -> Redirect {
    info!("battlenet callback received");
    let frontend_url = &state.config.frontend_url;

    if enforce_rate_limit(&state.rate_limiter, &headers, "bnet_callback", 20, 60)
        .await
        .is_err()
    {
        warn!("battlenet callback rate limited");
        return Redirect::to(&format!("{}/auth/callback?error=rate_limited", frontend_url));
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
        return Redirect::to(&format!(
            "{}/auth/callback?error=missing_params",
            frontend_url
        ));
    };

    match battlenet_service::handle_battlenet_redirect(&state, code, csrf_state).await {
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
                frontend_url, user_id,
            ))
        }
        Ok(BnetCallbackResult::RequiresEmail { pending_token, battletag }) => {
            info!("battlenet callback requires email completion");
            Redirect::to(&format!(
                "{}/auth/callback?needs_email=true&pending_token={}&battletag={}",
                frontend_url,
                urlencoding::encode(&pending_token),
                urlencoding::encode(&battletag),
            ))
        }
        Err(_err) => {
            error!("battlenet callback failed");
            Redirect::to(&format!(
                "{}/auth/callback?error=authentication_failed",
                frontend_url
            ))
        }
    }
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

pub async fn battlenet_connect_init(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<BnetConnectInitResponse> {
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    let url = battlenet_service::battlenet_connect_init_url(&state, user_id).await?;
    Ok(Json(BnetConnectInitResponse { url }))
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
