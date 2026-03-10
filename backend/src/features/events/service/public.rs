use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::events::models::{Event, EventsKpiResponse, ListEventsQuery, PaginatedEventsResponse},
    shared::errors::ApiError,
};

use super::repo;

pub async fn list_events_public(
    state: &AppState,
    viewer_user_id: Option<Uuid>,
    query: ListEventsQuery,
) -> Result<PaginatedEventsResponse, ApiError> {
    let normalized_search = query
        .search
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let normalized_type = query
        .event_type
        .map(|value| value.trim().to_uppercase())
        .filter(|value| matches!(value.as_str(), "PUG" | "TOURNEY"));

    let owner_only_user_id = match query.owner.as_deref() {
        Some("mine") => viewer_user_id,
        _ => None,
    };

    let sort = match query.sort.as_deref() {
        Some("newest") => repo::EventListSort::Newest,
        Some("players") => repo::EventListSort::Players,
        Some("name") => repo::EventListSort::Name,
        _ => repo::EventListSort::Soonest,
    };

    let normalized_per_page = query
        .limit
        .or(query.per_page)
        .unwrap_or(20)
        .clamp(1, 100);

    let normalized_page = if query.limit.is_some() {
        1
    } else {
        query.page.unwrap_or(1).max(1)
    };

    let offset = (normalized_page.saturating_sub(1)).saturating_mul(normalized_per_page);

    let options = repo::ListVisibleEventsOptions {
        search: normalized_search,
        event_type: normalized_type,
        owner_only_user_id,
        sort,
        limit: normalized_per_page,
        offset,
    };

    let listing = repo::list_visible_event_ids(&state.pool, options).await?;

    let mut events = Vec::with_capacity(listing.event_ids.len());
    for event_id in listing.event_ids {
        let mut event = repo::load_event(&state.pool, event_id).await?;
        event.is_owner = match viewer_user_id {
            Some(user_id) => repo::is_event_owner(&state.pool, event_id, user_id).await?,
            None => false,
        };
        events.push(event);
    }

    Ok(PaginatedEventsResponse {
        items: events,
        total: listing.total,
        page: normalized_page,
        per_page: normalized_per_page,
    })
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

pub async fn get_featured_event_public(
    state: &AppState,
    viewer_user_id: Option<Uuid>,
) -> Result<Option<Event>, ApiError> {
    let Some(event_id) = repo::featured_event_id(&state.pool).await? else {
        return Ok(None);
    };

    let mut event = repo::load_event(&state.pool, event_id).await?;
    event.is_owner = match viewer_user_id {
        Some(user_id) => repo::is_event_owner(&state.pool, event_id, user_id).await?,
        None => false,
    };

    Ok(Some(event))
}

pub async fn get_events_kpis_public(state: &AppState) -> Result<EventsKpiResponse, ApiError> {
    let kpis = repo::load_events_kpis(&state.pool).await?;

    Ok(EventsKpiResponse {
        total_events: if kpis.total_events < 0 { 0 } else { kpis.total_events as u64 },
        total_signups: if kpis.total_signups < 0 { 0 } else { kpis.total_signups as u64 },
        upcoming_events_this_week: if kpis.upcoming_events_this_week < 0 {
            0
        } else {
            kpis.upcoming_events_this_week as u64
        },
        upcoming_tourneys_this_week: if kpis.upcoming_tourneys_this_week < 0 {
            0
        } else {
            kpis.upcoming_tourneys_this_week as u64
        },
    })
}
