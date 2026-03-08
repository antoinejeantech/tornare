use uuid::Uuid;

use crate::{
    app::state::AppState,
    features::{
        events::models::{
            CreateEventSignupRequestInput, Event, EventSignupLinkResponse, EventSignupRequest,
            PublicEventSignupInfo,
        },
        permissions::require_event_owner_access,
    },
    shared::{
        errors::{bad_request, not_found, ApiError},
        models::MessageResponse,
    },
};

use super::{as_owner_event, ensure_event_exists, ensure_event_has_capacity_for_new_player, repo};
use super::validation::validate_signup_request_input;

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

pub async fn get_public_signup_info(
    state: &AppState,
    signup_token: &str,
) -> Result<PublicEventSignupInfo, ApiError> {
    let Some(info) = repo::event_signup_info_by_token(&state.pool, signup_token).await? else {
        return Err(not_found("Signup link not found"));
    };

    Ok(info)
}

pub async fn create_public_signup_request(
    state: &AppState,
    signup_token: &str,
    payload: CreateEventSignupRequestInput,
) -> Result<MessageResponse, ApiError> {
    validate_signup_request_input(&payload)?;

    let Some(info) = repo::event_signup_info_by_token(&state.pool, signup_token).await? else {
        return Err(not_found("Signup link not found"));
    };

    if info.current_players >= usize::from(info.max_players) {
        return Err(bad_request("Event roster is already full"));
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
        payload.role.trim(),
        payload.rank.trim(),
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
    require_event_owner_access(state, event_id, user_id).await?;

    let Some(request) = repo::get_signup_request(&state.pool, event_id, request_id).await? else {
        return Err(not_found("Signup request not found"));
    };

    if request.status != "pending" {
        return Err(bad_request("This signup request has already been reviewed"));
    }

    ensure_event_has_capacity_for_new_player(state, event_id).await?;

    repo::insert_event_player(&state.pool, event_id, &request.name, &request.role, &request.rank)
        .await?;

    let updated_count =
        repo::update_signup_request_status(&state.pool, event_id, request_id, "accepted").await?;
    if updated_count == 0 {
        return Err(bad_request("This signup request has already been reviewed"));
    }

    let event = repo::load_event(&state.pool, event_id).await?;
    Ok(as_owner_event(event))
}

pub async fn decline_signup_request_for_user(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    request_id: Uuid,
) -> Result<MessageResponse, ApiError> {
    require_event_owner_access(state, event_id, user_id).await?;

    let updated_count =
        repo::update_signup_request_status(&state.pool, event_id, request_id, "declined").await?;
    if updated_count == 0 {
        return Err(not_found("Pending signup request not found"));
    }

    Ok(MessageResponse {
        message: "Signup request declined".to_string(),
    })
}
