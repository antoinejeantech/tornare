use crate::{
    features::{
        events::models::{
            AddPlayerInput, CreateEventInput, CreateEventSignupRequestInput, EventFormat,
            EventType, UpdateEventInput, UpdateEventPlayerInput,
        },
        users::models::OVERWATCH_RANKS,
    },
    shared::errors::{bad_request, ApiError},
};

pub(super) fn validate_create_event_input(payload: &CreateEventInput) -> Result<(), ApiError> {
    let name = payload.name.trim();
    let description = payload.description.trim();

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

    if let Some(start_date) = normalize_optional_string(&payload.start_date) {
        if start_date.len() > 40 {
            return Err(bad_request("Event start date is too long"));
        }
    }

    if !(2..=99).contains(&payload.max_players) {
        return Err(bad_request("Max players must be between 2 and 99"));
    }

    match &payload.event_type {
        EventType::Pug => {
            if !matches!(&payload.format, EventFormat::FiveVFive | EventFormat::SixVSix) {
                return Err(bad_request("PUG events support only 5v5 or 6v6 format"));
            }
        }
        EventType::Tourney => {}
    }

    Ok(())
}

pub(super) fn validate_update_event_input(payload: &UpdateEventInput) -> Result<(), ApiError> {
    let create_shape = CreateEventInput {
        name: payload.name.clone(),
        description: payload.description.clone(),
        start_date: payload.start_date.clone(),
        event_type: payload.event_type.clone(),
        format: payload.format.clone(),
        max_players: payload.max_players,
    };

    validate_create_event_input(&create_shape)
}

pub(super) fn normalize_optional_string(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

pub(super) fn validate_add_player_input(payload: &AddPlayerInput) -> Result<(), ApiError> {
    let name = payload.name.trim();
    let role = payload.role.trim();
    let rank = payload.rank.trim();

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

pub(super) fn validate_event_player_update_input(
    payload: &UpdateEventPlayerInput,
) -> Result<(), ApiError> {
    let add_player_shape = AddPlayerInput {
        name: payload.name.clone(),
        role: payload.role.clone(),
        rank: payload.rank.clone(),
    };

    validate_add_player_input(&add_player_shape)
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
    let add_player_shape = AddPlayerInput {
        name: payload.name.clone(),
        role: payload.role.clone(),
        rank: payload.rank.clone(),
    };

    validate_add_player_input(&add_player_shape)
}
