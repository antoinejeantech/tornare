use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        auth::{models::AuthUser, require_authenticated_user_id},
        users::models::UpdateUserProfileInput,
    },
    shared::{
        errors::ApiResult,
    },
};

use super::service;

pub async fn get_user_profile(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<AuthUser> {
    service::get_user_profile_public(&state, user_id).await.map(Json)
}

pub async fn update_user_profile(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateUserProfileInput>,
) -> ApiResult<AuthUser> {
    let authenticated_user_id = require_authenticated_user_id(&state, &headers)?;
    service::update_user_profile_for_user(&state, authenticated_user_id, user_id, payload)
        .await
        .map(Json)
}
