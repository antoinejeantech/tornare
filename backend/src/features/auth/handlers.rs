use axum::{extract::State, http::HeaderMap, Json};

use crate::{
    app::{security::enforce_rate_limit, state::AppState},
    features::auth::models::{
        AuthResponse, AuthUser, LoginInput, LogoutInput, RefreshInput, RegisterInput,
    },
    shared::{
        errors::{ApiResult, forbidden},
        models::MessageResponse,
    },
};

use super::service;

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterInput>,
) -> ApiResult<AuthResponse> {
    if !state.config.public_signup_enabled {
        return Err(forbidden("Public signup is disabled for now. It will be available soon."));
    }

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
