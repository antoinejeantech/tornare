use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    shared::{
        errors::ApiResult,
        models::{Match, MessageResponse},
    },
};

use super::service;

pub async fn list_matches(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Vec<Match>> {
    service::list_matches_for_headers(&state, &headers).await.map(Json)
}

pub async fn get_match(
    Path(match_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Match> {
    service::get_match_for_headers(&state, &headers, match_id)
        .await
        .map(Json)
}

pub async fn delete_match(
    Path(match_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    service::delete_match_for_headers(&state, &headers, match_id)
        .await
        .map(Json)
}
