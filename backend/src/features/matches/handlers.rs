use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::auth::require_authenticated_user_id,
    shared::{
        errors::ApiResult,
        models::{Match, MessageResponse},
    },
};

use super::service;

pub async fn list_matches(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Vec<Match>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::list_matches_for_user(&state, user_id).await.map(Json)
}

pub async fn get_match(
    Path(match_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::get_match_for_user(&state, user_id, match_id)
        .await
        .map(Json)
}

pub async fn delete_match(
    Path(match_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    service::delete_match_for_user(&state, user_id, match_id)
        .await
        .map(Json)
}
