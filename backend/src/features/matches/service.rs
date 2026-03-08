use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::Match,
        permissions::{require_event_manage_access, require_event_view_access},
    },
    shared::{
        errors::{not_found, ApiError},
        models::MessageResponse,
    },
};

use super::repo;

pub async fn list_matches_for_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Match>, ApiError> {
    let match_ids = repo::list_visible_match_ids(&state.pool, user_id).await?;

    let mut matches = Vec::with_capacity(match_ids.len());
    for match_id in match_ids {
        matches.push(repo::load_match(&state.pool, match_id).await?);
    }

    Ok(matches)
}

pub async fn get_match_for_user(
    state: &AppState,
    user_id: Uuid,
    match_id: Uuid,
) -> Result<Match, ApiError> {
    let Some(event_id) = repo::get_match_event_id(&state.pool, match_id).await? else {
        return Err(not_found("Match not found"));
    };

    require_event_view_access(state, event_id, user_id).await?;
    repo::load_match(&state.pool, match_id).await
}

pub async fn delete_match_for_user(
    state: &AppState,
    user_id: Uuid,
    match_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    let Some(event_id) = repo::get_match_event_id(&state.pool, match_id).await? else {
        return Err(not_found("Match not found"));
    };

    require_event_manage_access(state, event_id, user_id).await?;

    let deleted = repo::delete_match_by_id(&state.pool, match_id).await?;
    if deleted == 0 {
        return Err(not_found("Match not found"));
    }

    Ok(MessageResponse {
        message: "Match deleted".to_string(),
    })
}
