use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{Event, EventsKpiResponse, ListEventsQuery, PaginatedEventsResponse},
        permissions::has_global_event_owner_access,
    },
    shared::errors::ApiError,
};

use super::repo;

/// Zero out contact handles and linked-account details on every player when
/// the viewer does not have manage access. Keeps id/username/display_name for
/// player-card display while keeping PII off the public read path.
fn strip_player_sensitive_fields(event: &mut Event) {
    for player in &mut event.players {
        if let Some(linked) = player.linked_user.as_mut() {
            linked.discord_username = None;
            linked.battletag = None;
            linked.avatar_url = None;
        }
        player.reported_discord = None;
        player.reported_battletag = None;
    }
}

async fn apply_event_access(
    state: &AppState,
    event: &mut Event,
    viewer_user_id: Option<Uuid>,
    has_global_manage_access: bool,
) -> Result<(), ApiError> {
    let Some(user_id) = viewer_user_id else {
        event.is_owner = false;
        event.can_manage = false;
        return Ok(());
    };

    event.is_owner = repo::is_event_owner(&state.pool, event.id, user_id).await?;
    event.can_manage = event.is_owner || has_global_manage_access;

    Ok(())
}

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

    let has_global_manage_access = match viewer_user_id {
        Some(user_id) => has_global_event_owner_access(state, user_id).await?,
        None => false,
    };

    let owner_only_user_id = match query.owner.as_deref() {
        Some("mine") => match viewer_user_id {
            Some(user_id) if !has_global_manage_access => Some(user_id),
            _ => None,
        },
        Some(other) => Uuid::parse_str(other).ok(),
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
        ended_only: query.ended_only.unwrap_or(false),
    };

    let listing = repo::list_visible_event_ids(&state.pool, options).await?;

    let mut events = Vec::with_capacity(listing.event_ids.len());
    for event_id in listing.event_ids {
        let mut event = repo::load_event(&state.pool, event_id).await?;
        apply_event_access(state, &mut event, viewer_user_id, has_global_manage_access).await?;
        if !event.can_manage {
            strip_player_sensitive_fields(&mut event);
        }
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
    let has_global_manage_access = match viewer_user_id {
        Some(user_id) => has_global_event_owner_access(state, user_id).await?,
        None => false,
    };
    apply_event_access(state, &mut event, viewer_user_id, has_global_manage_access).await?;
    if !event.can_manage {
        strip_player_sensitive_fields(&mut event);
    }
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
    let has_global_manage_access = match viewer_user_id {
        Some(user_id) => has_global_event_owner_access(state, user_id).await?,
        None => false,
    };
    apply_event_access(state, &mut event, viewer_user_id, has_global_manage_access).await?;
    if !event.can_manage {
        strip_player_sensitive_fields(&mut event);
    }

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
