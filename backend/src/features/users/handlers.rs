use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        auth::{models::AuthUser, require_authenticated_user_id},
        users::models::{SearchUsersQuery, UpdateUserProfileInput, UserSearchResult},
    },
    shared::{
        errors::ApiResult,
        models::MessageResponse,
    },
};

use super::service;

pub async fn search_users(
    State(state): State<AppState>,
    Query(params): Query<SearchUsersQuery>,
) -> ApiResult<Vec<UserSearchResult>> {
    let q = params.search.unwrap_or_default();
    let q = q.trim().to_string();
    service::search_users(&state, &q).await.map(Json)
}

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

pub async fn delete_user_account(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let authenticated_user_id = require_authenticated_user_id(&state, &headers)?;
    service::delete_user_account(&state, authenticated_user_id, user_id).await?;
    Ok(Json(MessageResponse { message: "Account deleted".to_string() }))
}
