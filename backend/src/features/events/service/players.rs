use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{AddPlayerInput, AssignEventPlayerTeamInput, Event, UpdateEventPlayerInput},
        permissions::require_event_owner_access,
    },
    shared::{
        errors::{not_found, ApiError},
        models::MessageResponse,
    },
};

use super::{
    ensure_event_has_capacity_for_new_player, repo,
};
use super::validation::{validate_add_player_input, validate_event_player_update_input};

pub async fn add_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: AddPlayerInput,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;
    validate_add_player_input(&payload)?;

    ensure_event_has_capacity_for_new_player(state, event_id).await?;

    repo::insert_event_player(
        &state.pool,
        event_id,
        payload.name.trim(),
        payload.role.trim(),
        payload.rank.trim(),
    )
    .await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn delete_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    player_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let deleted = repo::delete_event_player_by_id(&state.pool, event_id, player_id).await?;

    if !deleted {
        return Err(not_found("Player not found in this event"));
    }

    Ok(MessageResponse {
        message: "Player removed".to_string(),
    })
}

pub async fn update_event_player_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    player_id: Uuid,
    payload: UpdateEventPlayerInput,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;
    validate_event_player_update_input(&payload)?;

    let updated = repo::update_event_player_by_id(
        &state.pool,
        event_id,
        player_id,
        payload.name.trim(),
        payload.role.trim(),
        payload.rank.trim(),
    )
    .await?;

    if !updated {
        return Err(not_found("Player not found in this event"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn assign_event_player_team_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: AssignEventPlayerTeamInput,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    if !repo::event_player_exists(&state.pool, event_id, payload.player_id).await? {
        return Err(not_found("Player not found in this event"));
    }

    if let Some(team_id) = payload.team_id {
        if !repo::event_team_exists(&state.pool, event_id, team_id).await? {
            return Err(not_found("Team not found in this event"));
        }

        repo::upsert_event_player_team_membership(&state.pool, event_id, team_id, payload.player_id)
            .await?;
    } else {
        repo::delete_event_player_team_membership(&state.pool, event_id, payload.player_id)
            .await?;
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}
