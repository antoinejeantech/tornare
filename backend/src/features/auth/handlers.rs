use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;

use crate::{
    app::{security::enforce_rate_limit, state::AppState},
    features::auth::models::{
        AuthResponse, AuthUser, ForgotPasswordInput, LoginInput, LogoutInput,
        PendingVerificationResponse, RefreshInput, RegisterInput, ResendVerificationInput,
        ResetPasswordInput,
    },
    shared::{
        errors::ApiResult,
        models::MessageResponse,
    },
};

use super::service;

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterInput>,
) -> ApiResult<PendingVerificationResponse> {
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

pub async fn verify_email(
    State(state): State<AppState>,
    Query(params): Query<VerifyEmailQuery>,
) -> ApiResult<AuthResponse> {
    service::verify_email(&state, &params.token).await.map(Json)
}

pub async fn resend_verification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ResendVerificationInput>,
) -> ApiResult<MessageResponse> {
    enforce_rate_limit(&state.rate_limiter, &headers, "resend_verification", 5, 60).await?;
    service::resend_verification(&state, &payload.email).await.map(Json)
}

pub async fn forgot_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ForgotPasswordInput>,
) -> ApiResult<MessageResponse> {
    enforce_rate_limit(&state.rate_limiter, &headers, "forgot_password", 5, 60).await?;
    service::forgot_password(&state, payload).await.map(Json)
}

pub async fn reset_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ResetPasswordInput>,
) -> ApiResult<MessageResponse> {
    enforce_rate_limit(&state.rate_limiter, &headers, "reset_password", 10, 60).await?;
    service::reset_password(&state, payload).await.map(Json)
}
