use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        auth::service::maybe_authenticated_user_id,
        events::models::{
            CreateEventSignupRequestInput, Event, EventSignupLinkResponse,
            EventSignupRequest, PublicEventSignupInfo, SignupStatus,
        },
        permissions::require_event_owner_access,
    },
    shared::{
        errors::{bad_request, internal_error, not_found, ApiError},
        models::MessageResponse,
    },
};

use axum::http::HeaderMap;

use super::{ensure_event_exists, ensure_event_has_capacity_for_new_player, repo};
use super::validation::validate_signup_request_input;

pub const MAX_SIGNUP_REQUESTS_PER_EVENT: usize = 99;

pub async fn get_event_signup_link_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<EventSignupLinkResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let Some(signup_token) = repo::signup_token_for_event(&state.pool, event_id).await? else {
        return Err(not_found("Event not found"));
    };

    Ok(EventSignupLinkResponse { signup_token })
}

pub async fn rotate_event_signup_link_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<EventSignupLinkResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let new_token = Uuid::new_v4().to_string();
    let updated = repo::rotate_signup_token_for_event(&state.pool, event_id, &new_token).await?;

    if !updated {
        return Err(not_found("Event not found"));
    }

    Ok(EventSignupLinkResponse {
        signup_token: new_token,
    })
}

pub async fn set_event_public_signup_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    enabled: bool,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;
    let current_event = repo::load_event(&state.pool, event_id).await?;
    let should_rotate_token = current_event.public_signup_enabled && !enabled;
    let signup_token = if should_rotate_token {
        Some(Uuid::new_v4().to_string())
    } else {
        None
    };

    let updated = repo::set_public_signup_enabled_for_event(
        &state.pool,
        event_id,
        enabled,
        signup_token.as_deref(),
    )
    .await?;

    if !updated {
        return Err(not_found("Event not found"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn get_public_signup_info(
    state: &AppState,
    signup_token: &str,
) -> Result<PublicEventSignupInfo, ApiError> {
    let token = signup_token.trim();
    let Some(info) = repo::event_signup_info_by_token(&state.pool, token).await? else {
        return Err(not_found("Signup link not found"));
    };

    Ok(info)
}

pub async fn create_public_signup_request(
    state: &AppState,
    signup_token: &str,
    payload: CreateEventSignupRequestInput,
    headers: &HeaderMap,
) -> Result<MessageResponse, ApiError> {
    validate_signup_request_input(&payload)?;

    let submitter_user_id = maybe_authenticated_user_id(state, headers);

    let token = signup_token.trim();
    let Some(info) = repo::event_signup_info_by_token(&state.pool, token).await? else {
        return Err(not_found("Signup link not found"));
    };

    if info.current_signup_requests >= MAX_SIGNUP_REQUESTS_PER_EVENT {
        return Err(bad_request("Signup request limit reached for this event"));
    }

    let clean_name = payload.name.trim();
    if repo::has_pending_signup_request_with_name(&state.pool, info.event_id, clean_name).await? {
        return Err(bad_request(
            "A signup request with this name is already pending",
        ));
    }

    repo::create_signup_request(
        &state.pool,
        info.event_id,
        clean_name,
        submitter_user_id,
        &payload.roles,
        payload.discord_username.as_deref().map(str::trim).filter(|s| !s.is_empty()),
        payload.battletag.as_deref().map(str::trim).filter(|s| !s.is_empty()),
    )
    .await?;

    Ok(MessageResponse {
        message: "Signup request sent".to_string(),
    })
}

pub async fn list_signup_requests_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Vec<EventSignupRequest>, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;
    ensure_event_exists(state, event_id).await?;

    repo::list_signup_requests_for_event(&state.pool, event_id).await
}

pub async fn accept_signup_request_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    request_id: Uuid,
) -> Result<Event, ApiError> {
    let is_owner = require_event_owner_access(state, event_id, user_id).await?;

    let Some(request) = repo::get_signup_request(&state.pool, event_id, request_id).await? else {
        return Err(not_found("Signup request not found"));
    };

    if request.status != SignupStatus::Pending {
        return Err(bad_request("This signup request has already been reviewed"));
    }

    ensure_event_has_capacity_for_new_player(state, event_id).await?;

    let primary = request
        .roles
        .first()
        .ok_or_else(|| bad_request("Signup request has no role preferences"))?;

    let role_pairs: Vec<(&str, &str)> = request
        .roles
        .iter()
        .map(|r| (r.role.as_db_value(), r.rank.as_db_value()))
        .collect();

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    // Atomically claim the request; a concurrent accept for the same request gets 0 rows.
    let claimed = repo::update_signup_request_status_in_tx(
        &mut tx,
        event_id,
        request_id,
        SignupStatus::Accepted.as_db_value(),
    )
    .await?;
    if claimed == 0 {
        return Err(bad_request("This signup request has already been reviewed"));
    }

    repo::insert_event_player_in_tx(
        &mut tx,
        event_id,
        &request.name,
        primary.role.as_db_value(),
        primary.rank.as_db_value(),
        Some(request_id),
        request.submitter_user_id,
        &role_pairs,
    )
    .await?;

    tx.commit().await.map_err(internal_error)?;

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(event.into_owner(is_owner))
}

pub async fn decline_signup_request_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    request_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let updated_count = repo::update_signup_request_status(
        &state.pool,
        event_id,
        request_id,
        SignupStatus::Declined.as_db_value(),
    )
    .await?;
    if updated_count == 0 {
        return Err(not_found("Pending signup request not found"));
    }

    Ok(MessageResponse {
        message: "Signup request declined".to_string(),
    })
}
