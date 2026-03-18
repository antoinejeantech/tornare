use crate::{
    features::events::models::{
        AddPlayerInput, CreateEventInput, CreateEventSignupRequestInput, UpdateEventInput,
        UpdateEventPlayerInput,
    },
    shared::errors::ApiError,
    shared::validation::normalize_optional_rfc3339_timestamp,
};
use time::OffsetDateTime;

// These thin wrappers exist so service code can keep calling the same function
// names they used before. They simply delegate to the validate() methods that
// live on each DTO, keeping the validation logic as the single authority.

pub(super) fn validate_create_event_input(payload: &CreateEventInput) -> Result<(), ApiError> {
    payload.validate()
}

pub(super) fn validate_update_event_input(payload: &UpdateEventInput) -> Result<(), ApiError> {
    payload.validate()
}

pub(super) fn validate_add_player_input(payload: &AddPlayerInput) -> Result<(), ApiError> {
    payload.validate()
}

pub(super) fn validate_event_player_update_input(
    payload: &UpdateEventPlayerInput,
) -> Result<(), ApiError> {
    payload.validate()
}

pub(super) fn validate_signup_request_input(
    payload: &CreateEventSignupRequestInput,
) -> Result<(), ApiError> {
    payload.validate()
}

pub(super) fn validate_event_team_name(name: &str) -> Result<(), ApiError> {
    use crate::shared::errors::bad_request;
    if name.trim().is_empty() {
        return Err(bad_request("Team name is required"));
    }
    Ok(())
}

// Utilities used by services for date parsing.

pub(super) fn normalize_optional_start_date(
    value: &Option<String>,
) -> Result<Option<OffsetDateTime>, ApiError> {
    normalize_optional_rfc3339_timestamp(value.as_deref())
}

