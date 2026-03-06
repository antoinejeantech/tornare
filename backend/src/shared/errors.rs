use axum::{http::StatusCode, Json};

use crate::shared::models::ErrorResponse;

pub type ApiError = (StatusCode, Json<ErrorResponse>);
pub type ApiResult<T> = Result<Json<T>, ApiError>;

pub fn bad_request(message: &str) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

pub fn unauthorized(message: &str) -> ApiError {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

pub fn forbidden(message: &str) -> ApiError {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

pub fn too_many_requests(message: &str) -> ApiError {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

pub fn not_found(message: &str) -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

pub fn internal_error(error: sqlx::Error) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: format!("Database query failed: {error}"),
        }),
    )
}
