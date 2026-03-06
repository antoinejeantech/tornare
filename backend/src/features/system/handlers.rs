use axum::Json;

use crate::shared::models::MessageResponse;

use super::service;

pub async fn health() -> &'static str {
    service::health().await
}

pub async fn hello() -> Json<MessageResponse> {
    Json(service::hello_message())
}
