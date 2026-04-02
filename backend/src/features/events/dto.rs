use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::domain::{
    BracketGenerationMode, Event, EventFormat, EventType, PlayerRank, PlayerRole,
};
use crate::shared::{
    errors::{bad_request, ApiError},
    serde_utils::NullableField,
    validation::parse_rfc3339_timestamp,
};

// ---------------------------------------------------------------------------
// Helpers shared by event/player validation
// ---------------------------------------------------------------------------

pub(super) fn normalize_optional_string(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

// ---------------------------------------------------------------------------
// Event input DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateEventInput {
    pub name: String,
    pub description: String,
    pub start_date: Option<String>,
    pub event_type: EventType,
    pub format: EventFormat,
    #[serde(default)]
    pub public_signup_enabled: bool,
    pub max_players: u8,
    #[serde(default)]
    pub require_discord: bool,
    #[serde(default)]
    pub require_battletag: bool,
}

impl CreateEventInput {
    pub fn validate(&self) -> Result<(), ApiError> {
        validate_event_fields(
            &self.name,
            &self.description,
            &self.start_date,
            self.max_players,
            &self.event_type,
            &self.format,
        )
    }
}

#[derive(Deserialize)]
pub struct UpdateEventInput {
    pub name: String,
    pub description: String,
    pub start_date: Option<String>,
    pub event_type: EventType,
    pub format: EventFormat,
    pub max_players: u8,
    #[serde(default)]
    pub require_discord: bool,
    #[serde(default)]
    pub require_battletag: bool,
}

impl UpdateEventInput {
    pub fn validate(&self) -> Result<(), ApiError> {
        validate_event_fields(
            &self.name,
            &self.description,
            &self.start_date,
            self.max_players,
            &self.event_type,
            &self.format,
        )
    }
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

// ---------------------------------------------------------------------------
// Player input DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddPlayerInput {
    pub name: String,
    pub role: String,
    pub rank: String,
}

impl AddPlayerInput {
    pub fn validate(&self) -> Result<(), ApiError> {
        validate_player_fields(&self.name, &self.role, &self.rank)
    }
}

#[derive(Deserialize)]
pub struct UpdateEventPlayerInput {
    pub name: String,
    pub role: String,
    pub rank: String,
    /// When provided, replaces the player's role preferences (used by auto-balance).
    /// `roles[0]` also becomes the player's primary role/rank.
    pub roles: Option<Vec<RolePreferenceInput>>,
}

impl UpdateEventPlayerInput {
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(roles) = &self.roles {
            if roles.is_empty() {
                return Err(bad_request("At least one role preference is required"));
            }
            if roles.len() > 3 {
                return Err(bad_request("At most 3 role preferences are allowed"));
            }
            let mut seen = std::collections::HashSet::new();
            for rp in roles {
                PlayerRole::try_from(rp.role.trim())
                    .map_err(|_| bad_request("Role must be Tank, DPS, or Support"))?;
                PlayerRank::try_from(rp.rank.trim())
                    .map_err(|_| bad_request("Invalid player rank"))?;
                if !seen.insert(rp.role.trim().to_lowercase()) {
                    return Err(bad_request("Duplicate role preferences are not allowed"));
                }
            }
            // Validate the name using roles[0] as primary
            let primary = &roles[0];
            validate_player_fields(&self.name, &primary.role, &primary.rank)
        } else {
            validate_player_fields(&self.name, &self.role, &self.rank)
        }
    }
}

/// A single role+rank preference as submitted by an applicant.
#[derive(Deserialize)]
pub struct RolePreferenceInput {
    pub role: String,
    pub rank: String,
}

#[derive(Deserialize)]
pub struct CreateEventSignupRequestInput {
    pub name: String,
    pub roles: Vec<RolePreferenceInput>,
    pub discord_username: Option<String>,
    pub battletag: Option<String>,
}

impl CreateEventSignupRequestInput {
    pub fn validate(&self) -> Result<(), ApiError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(bad_request("Player name is required"));
        }
        if name.len() > 60 {
            return Err(bad_request("Player name must be 60 characters or fewer"));
        }
        if self.roles.is_empty() {
            return Err(bad_request("At least one role preference is required"));
        }
        if self.roles.len() > 3 {
            return Err(bad_request("At most 3 role preferences are allowed"));
        }
        let mut seen_roles = std::collections::HashSet::new();
        for entry in &self.roles {
            let role = entry.role.trim();
            let rank = entry.rank.trim();
            PlayerRole::try_from(role)
                .map_err(|_| bad_request("Role must be Tank, DPS, or Support"))?;
            PlayerRank::try_from(rank)
                .map_err(|_| bad_request("Invalid player rank"))?;
            if !seen_roles.insert(role.to_string()) {
                return Err(bad_request("Duplicate role preferences are not allowed"));
            }
        }
        if let Some(ref d) = self.discord_username {
            if d.trim().len() > 100 {
                return Err(bad_request("Discord username must be 100 characters or fewer"));
            }
        }
        if let Some(ref b) = self.battletag {
            if b.trim().len() > 100 {
                return Err(bad_request("Battletag must be 100 characters or fewer"));
            }
        }
        Ok(())
    }
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

    PlayerRole::try_from(role)
        .map_err(|_| bad_request("Role must be Tank, DPS, or Support"))?;

    PlayerRank::try_from(rank)
        .map_err(|_| bad_request("Invalid player rank"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Team input DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateEventTeamInput {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateEventTeamInput {
    pub name: String,
}

// ---------------------------------------------------------------------------
// Assignment / matchup input DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssignEventPlayerTeamInput {
    pub player_id: Uuid,
    pub team_id: Option<Uuid>,
    pub assigned_role: Option<String>,
    pub assigned_rank: Option<String>,
}

#[derive(Deserialize)]
pub struct SetMatchupInput {
    #[serde(default)]
    pub team_a_id: NullableField<Uuid>,
    #[serde(default)]
    pub team_b_id: NullableField<Uuid>,
}

#[derive(Deserialize)]
pub struct ReportMatchWinnerInput {
    pub winner_team_id: Uuid,
}

// ---------------------------------------------------------------------------
// Match input DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateEventMatchInput {
    pub title: String,
    pub map: String,
    pub start_date: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateMatchStartDateInput {
    pub start_date: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateMatchInput {
    pub title: String,
    pub map: String,
    pub max_players: u8,
    pub start_date: Option<String>,
}

// ---------------------------------------------------------------------------
// Bracket input DTOs
// ---------------------------------------------------------------------------

fn default_bracket_generation_mode() -> BracketGenerationMode {
    BracketGenerationMode::Random
}

#[derive(Deserialize)]
pub struct GenerateTourneyBracketInput {
    #[serde(default = "default_bracket_generation_mode")]
    pub mode: BracketGenerationMode,
}

// ---------------------------------------------------------------------------
// Event flag input DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SetEventPublicSignupInput {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct SetEventFeaturedInput {
    pub featured: bool,
}

// ---------------------------------------------------------------------------
// Query / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
pub struct ListEventsQuery {
    pub search: Option<String>,
    pub owner: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub limit: Option<u32>,
    /// Filter by event status. None / "all" shows ACTIVE+ENDED; "active", "ended", "draft" filter to that status.
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedEventsResponse {
    pub items: Vec<Event>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Serialize)]
pub struct EventsKpiResponse {
    pub total_events: u64,
    pub total_signups: u64,
    pub upcoming_events_this_week: u64,
    pub upcoming_tourneys_this_week: u64,
}

#[derive(Serialize)]
pub struct EventSignupLinkResponse {
    pub signup_token: String,
}

#[derive(Serialize)]
pub struct AutoBalanceTeamsResponse {
    pub event: Event,
    pub summary: String,
}


