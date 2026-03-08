use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::events::models::Event,
    shared::errors::ApiError,
};

use super::repo;

pub async fn list_events_public(
    state: &AppState,
    viewer_user_id: Option<Uuid>,
) -> Result<Vec<Event>, ApiError> {
    let event_ids = repo::list_visible_event_ids(&state.pool).await?;

    let mut events = Vec::with_capacity(event_ids.len());
    for event_id in event_ids {
        let mut event = repo::load_event(&state.pool, event_id).await?;
        event.is_owner = match viewer_user_id {
            Some(user_id) => repo::is_event_owner(&state.pool, event_id, user_id).await?,
            None => false,
        };
        events.push(event);
    }

    Ok(events)
}

pub async fn get_event_public(
    state: &AppState,
    event_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<Event, ApiError> {
    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = match viewer_user_id {
        Some(user_id) => repo::is_event_owner(&state.pool, event_id, user_id).await?,
        None => false,
    };
    Ok(event)
}
