use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Redirect,
    Json,
};
use serde::Deserialize;

use crate::{
    app::{security::enforce_rate_limit, state::AppState},
    features::auth::models::{
        AuthResponse, AuthUser, BnetConnectInitResponse, LoginInput, LogoutInput, RefreshInput,
        RegisterInput,
    },
    shared::{
        errors::ApiResult,
        models::MessageResponse,
    },
};

use super::service::{self, BnetCallbackResult};

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterInput>,
) -> ApiResult<AuthResponse> {
    enforce_rate_limit(&state.rate_limiter, &headers, "auth_register", 10, 60).await?;
    service::register_user(&state, payload).await.map(Json)
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginInput>,
) -> ApiResult<AuthResponse> {
    enforce_rate_limit(&state.rate_limiter, &headers, "auth_login", 20, 60).await?;
    service::login_user(&state, payload).await.map(Json)
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<AuthUser> {
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    let user = service::get_auth_user_by_id(&state, user_id).await?;
    Ok(Json(user))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RefreshInput>,
) -> ApiResult<AuthResponse> {
    enforce_rate_limit(&state.rate_limiter, &headers, "auth_refresh", 30, 60).await?;
    service::refresh_auth(&state, &payload.refresh_token).await.map(Json)
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutInput>,
) -> ApiResult<MessageResponse> {
    service::logout_session(&state, &payload.refresh_token).await?;
    Ok(Json(MessageResponse {
        message: "Logged out".to_string(),
    }))
}

pub async fn battlenet_authorize(State(state): State<AppState>) -> Redirect {
    if state.config.battlenet_client_id.is_empty() {
        return Redirect::to(&format!(
            "{}/auth/callback?error=oauth_not_configured",
            state.config.frontend_url
        ));
    }

    let csrf_state = service::build_oauth_state(&state.config.jwt_secret);
    let url = format!(
        "https://oauth.battle.net/authorize?client_id={}&scope=openid%20email&state={}&redirect_uri={}&response_type=code",
        urlencoding::encode(&state.config.battlenet_client_id),
        urlencoding::encode(&csrf_state),
        urlencoding::encode(&state.config.battlenet_redirect_uri),
    );
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
    Query(params): Query<BnetCallbackParams>,
) -> Redirect {
    let frontend_url = &state.config.frontend_url;

    if let Some(error) = &params.error {
        return Redirect::to(&format!(
            "{}/auth/callback?error={}",
            frontend_url,
            urlencoding::encode(error),
        ));
    }

    let (Some(code), Some(csrf_state)) =
        (params.code.as_deref(), params.state.as_deref())
    else {
        return Redirect::to(&format!(
            "{}/auth/callback?error=missing_params",
            frontend_url
        ));
    };

    match service::handle_battlenet_redirect(&state, code, csrf_state).await {
        Ok(BnetCallbackResult::LoggedIn(auth)) => Redirect::to(&format!(
            "{}/auth/callback?access_token={}&refresh_token={}",
            frontend_url,
            urlencoding::encode(&auth.access_token),
            urlencoding::encode(&auth.refresh_token),
        )),
        Ok(BnetCallbackResult::Connected(user_id)) => Redirect::to(&format!(
            "{}/auth/callback?connected=true&profile_id={}",
            frontend_url, user_id,
        )),
        Err(_) => Redirect::to(&format!(
            "{}/auth/callback?error=authentication_failed",
            frontend_url
        )),
    }
}

pub async fn battlenet_connect_init(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<BnetConnectInitResponse> {
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    let url = service::battlenet_connect_init_url(&state, user_id).await?;
    Ok(Json(BnetConnectInitResponse { url }))
}

pub async fn battlenet_disconnect(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    service::disconnect_battlenet(&state, user_id).await?;
    Ok(Json(MessageResponse {
        message: "Battle.net account disconnected".to_string(),
    }))
}
