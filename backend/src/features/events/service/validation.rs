use crate::{
    features::{
        events::models::{
            AddPlayerInput, CreateEventInput, CreateEventSignupRequestInput, EventFormat,
            EventType, UpdateEventInput, UpdateEventPlayerInput,
        },
        users::models::OVERWATCH_RANKS,
    },
    shared::{
        errors::{bad_request, ApiError},
        validation::{normalize_optional_rfc3339_timestamp, parse_rfc3339_timestamp},
    },
};
use time::OffsetDateTime;

pub(super) fn validate_create_event_input(payload: &CreateEventInput) -> Result<(), ApiError> {
    validate_event_fields(
        &payload.name,
        &payload.description,
        &payload.start_date,
        payload.max_players,
        &payload.event_type,
        &payload.format,
    )
}

pub(super) fn validate_update_event_input(payload: &UpdateEventInput) -> Result<(), ApiError> {
    validate_event_fields(
        &payload.name,
        &payload.description,
        &payload.start_date,
        payload.max_players,
        &payload.event_type,
        &payload.format,
    )
}

fn validate_event_fields(
    name: &str,
    description: &str,
    start_date: &Option<String>,
    max_players: u8,
    event_type: &EventType,
    format: &EventFormat,
) -> Result<(), ApiError> {
    let name = name.trim();
    let description = description.trim();

    if name.is_empty() {
        return Err(bad_request("Event name is required"));
    }

    if name.len() > 120 {
        return Err(bad_request("Event name must be 120 characters or fewer"));
    }

    if description.len() > 5000 {
        return Err(bad_request(
            "Event description must be 5000 characters or fewer",
        ));
    }

    if let Some(start_date) = normalize_optional_string(start_date) {
        if start_date.len() > 64 {
            return Err(bad_request("Event start date is too long"));
        }

        parse_rfc3339_timestamp(start_date.as_str())?;
    }

    if !(2..=99).contains(&max_players) {
        return Err(bad_request("Max players must be between 2 and 99"));
    }

    match event_type {
        EventType::Pug => {
            if !matches!(format, EventFormat::FiveVFive | EventFormat::SixVSix) {
                return Err(bad_request("PUG events support only 5v5 or 6v6 format"));
            }
        }
        EventType::Tourney => {}
    }

    Ok(())
}

pub(super) fn normalize_optional_string(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

pub(super) fn normalize_optional_start_date(
    value: &Option<String>,
) -> Result<Option<OffsetDateTime>, ApiError> {
    normalize_optional_rfc3339_timestamp(value.as_deref())
}

pub(super) fn validate_add_player_input(payload: &AddPlayerInput) -> Result<(), ApiError> {
    validate_player_fields(&payload.name, &payload.role, &payload.rank)
}

pub(super) fn validate_event_player_update_input(
    payload: &UpdateEventPlayerInput,
) -> Result<(), ApiError> {
    validate_player_fields(&payload.name, &payload.role, &payload.rank)
}

pub(super) fn validate_event_team_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(bad_request("Team name is required"));
    }

    Ok(())
}

pub(super) fn validate_signup_request_input(
    payload: &CreateEventSignupRequestInput,
) -> Result<(), ApiError> {
    validate_player_fields(&payload.name, &payload.role, &payload.rank)
}

fn validate_player_fields(name: &str, role: &str, rank: &str) -> Result<(), ApiError> {
    let name = name.trim();
    let role = role.trim();
    let rank = rank.trim();

    if name.is_empty() {
        return Err(bad_request("Player name is required"));
    }

    if name.len() > 60 {
        return Err(bad_request("Player name must be 60 characters or fewer"));
    }

    if role.is_empty() {
        return Err(bad_request("Player role is required"));
    }

    if !matches!(role, "Tank" | "DPS" | "Support") {
        return Err(bad_request("Role must be Tank, DPS, or Support"));
    }

    if rank.is_empty() {
        return Err(bad_request("Player rank is required"));
    }

    if !OVERWATCH_RANKS.contains(&rank) {
        return Err(bad_request("Invalid player rank"));
    }

    Ok(())
}
