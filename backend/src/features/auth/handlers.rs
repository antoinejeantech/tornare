use axum::{extract::State, http::HeaderMap, Json};

use crate::{
    app::state::AppState,
    shared::{
        errors::ApiResult,
        models::{AuthResponse, AuthUser, LoginInput, LogoutInput, MessageResponse, RefreshInput, RegisterInput},
    },
};

use super::service;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterInput>,
) -> ApiResult<AuthResponse> {
    service::register_user(&state, payload).await.map(Json)
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginInput>,
) -> ApiResult<AuthResponse> {
    service::login_user(&state, payload).await.map(Json)
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<AuthUser> {
    let user_id = service::require_authenticated_user_id(&state, &headers)?;
    let user = service::get_auth_user_by_id(&state, user_id).await?;
    Ok(Json(user))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshInput>,
) -> ApiResult<AuthResponse> {
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
