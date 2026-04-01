use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{CreateEventInput, Event, UpdateEventInput},
        permissions::{require_event_admin_access, require_event_owner_access},
    },
    shared::{
        errors::{conflict, not_found, ApiError},
        models::MessageResponse,
    },
};

use super::repo;
use super::validation::{
    normalize_optional_start_date, validate_create_event_input, validate_update_event_input,
};

pub async fn create_event_for_user(
    state: &AppState,
    user_id: Uuid,
    payload: CreateEventInput,
) -> Result<Event, ApiError> {
    validate_create_event_input(&payload)?;

    let event_id = Uuid::new_v4();
    let signup_token = Uuid::new_v4().to_string();
    let normalized_start_date = normalize_optional_start_date(&payload.start_date)?;

    repo::insert_event(
        &state.pool,
        event_id,
        payload.name.trim(),
        payload.description.trim(),
        normalized_start_date,
        payload.event_type.as_db_value(),
        payload.format.as_db_value(),
        payload.public_signup_enabled,
        i32::from(payload.max_players),
        &signup_token,
    )
    .await?;

    repo::insert_event_owner_membership(&state.pool, event_id, user_id).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(true))
}

pub async fn update_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: UpdateEventInput,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;
    validate_update_event_input(&payload)?;
    let normalized_start_date = normalize_optional_start_date(&payload.start_date)?;

    let updated = repo::update_event_details(
        &state.pool,
        event_id,
        payload.name.trim(),
        payload.description.trim(),
        normalized_start_date,
        payload.event_type.as_db_value(),
        payload.format.as_db_value(),
        i32::from(payload.max_players),
    )
    .await?;

    if !updated {
        return Err(not_found("Event not found"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn delete_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let deleted = repo::delete_event_by_id(&state.pool, event_id).await?;

    if deleted == 0 {
        return Err(not_found("Event not found"));
    }

    Ok(MessageResponse {
        message: "Event deleted".to_string(),
    })
}

pub async fn set_featured_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    featured: bool,
) -> Result<Event, ApiError> {
    require_event_admin_access(state, user_id).await?;

    if !repo::event_exists(&state.pool, event_id).await? {
        return Err(not_found("Event not found"));
    }

    repo::set_featured_event_state(&state.pool, event_id, featured).await?;

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = repo::is_event_owner(&state.pool, event_id, user_id).await?;
    event.can_manage = true;
    Ok(event)
}

pub async fn publish_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    let new_status = event.status.publish().map_err(conflict)?;
    repo::set_event_status(&state.pool, event_id, new_status).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn unpublish_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    let new_status = event.status.unpublish().map_err(conflict)?;
    repo::set_event_status(&state.pool, event_id, new_status).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn end_event_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    let new_status = event.status.end().map_err(conflict)?;
    repo::set_event_status(&state.pool, event_id, new_status).await?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}
