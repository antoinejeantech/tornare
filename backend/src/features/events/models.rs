use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_bracket_generation_mode() -> BracketGenerationMode {
    BracketGenerationMode::Random
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum EventType {
    Pug,
    Tourney,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EventFormat {
    #[serde(rename = "5v5")]
    FiveVFive,
    #[serde(rename = "6v6")]
    SixVSix,
    #[serde(rename = "1v1")]
    OneVOne,
}

impl EventFormat {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            EventFormat::FiveVFive => "5v5",
            EventFormat::SixVSix => "6v6",
            EventFormat::OneVOne => "1v1",
        }
    }
}

impl TryFrom<&str> for EventFormat {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "5v5" => Ok(EventFormat::FiveVFive),
            "6v6" => Ok(EventFormat::SixVSix),
            "1v1" => Ok(EventFormat::OneVOne),
            other => Err(format!("Invalid event format value in database: {other}")),
        }
    }
}

impl EventType {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            EventType::Pug => "PUG",
            EventType::Tourney => "TOURNEY",
        }
    }
}

impl TryFrom<&str> for EventType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "PUG" => Ok(EventType::Pug),
            "TOURNEY" => Ok(EventType::Tourney),
            other => Err(format!("Invalid event type value in database: {other}")),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub rank: String,
    pub team_id: Option<Uuid>,
    pub team: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Match {
    pub id: Uuid,
    pub event_id: Option<Uuid>,
    pub team_a_id: Option<Uuid>,
    pub team_a_name: Option<String>,
    pub team_b_id: Option<Uuid>,
    pub team_b_name: Option<String>,
    pub title: String,
    pub map: String,
    pub max_players: u8,
    pub round: Option<i32>,
    pub position: Option<i32>,
    pub next_match_id: Option<Uuid>,
    pub next_match_slot: Option<String>,
    pub winner_team_id: Option<Uuid>,
    pub winner_team_name: Option<String>,
    pub is_bracket: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub start_date: Option<String>,
    pub players: Vec<Player>,
}

#[derive(Serialize, Clone)]
pub struct EventTeam {
    pub id: Uuid,
    pub name: String,
    pub player_ids: Vec<Uuid>,
}

#[derive(Serialize, Clone)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub start_date: Option<String>,
    pub event_type: EventType,
    pub format: EventFormat,
    pub is_featured: bool,
    pub is_owner: bool,
    pub creator_id: Option<Uuid>,
    pub creator_name: Option<String>,
    pub public_signup_enabled: bool,
    pub public_signup_token: Option<String>,
    pub max_players: u8,
    pub players: Vec<Player>,
    pub teams: Vec<EventTeam>,
    pub matches: Vec<Match>,
}

#[derive(Deserialize)]
pub struct SetEventPublicSignupInput {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct SetEventFeaturedInput {
    pub featured: bool,
}

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
}

#[derive(Deserialize)]
pub struct UpdateEventInput {
    pub name: String,
    pub description: String,
    pub start_date: Option<String>,
    pub event_type: EventType,
    pub format: EventFormat,
    pub max_players: u8,
}

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
pub struct CreateEventTeamInput {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateEventTeamInput {
    pub name: String,
}

#[derive(Deserialize)]
pub struct AssignEventPlayerTeamInput {
    pub player_id: Uuid,
    pub team_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct SetMatchupInput {
    pub team_a_id: Option<Uuid>,
    pub team_b_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct ReportMatchWinnerInput {
    pub winner_team_id: Uuid,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BracketGenerationMode {
    Random,
    Empty,
}

#[derive(Deserialize)]
pub struct GenerateTourneyBracketInput {
    #[serde(default = "default_bracket_generation_mode")]
    pub mode: BracketGenerationMode,
}

#[derive(Deserialize)]
pub struct CreateMatchInput {
    pub title: String,
    pub map: String,
    pub max_players: u8,
    pub start_date: Option<String>,
}

#[derive(Deserialize)]
pub struct AddPlayerInput {
    pub name: String,
    pub role: String,
    pub rank: String,
}

#[derive(Deserialize)]
pub struct UpdateEventPlayerInput {
    pub name: String,
    pub role: String,
    pub rank: String,
}

#[derive(Deserialize)]
pub struct CreateEventSignupRequestInput {
    pub name: String,
    pub role: String,
    pub rank: String,
}

#[derive(Serialize)]
pub struct EventSignupRequest {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub role: String,
    pub rank: String,
    pub status: String,
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

#[derive(Serialize)]
pub struct PublicEventSignupInfo {
    pub event_id: Uuid,
    pub event_name: String,
    pub event_description: String,
    pub start_date: Option<String>,
    pub event_type: EventType,
    pub format: EventFormat,
    pub max_players: u8,
    pub current_players: usize,
    pub current_signup_requests: usize,
}

#[cfg(test)]
mod tests {
    use super::{EventFormat, EventType};

    #[test]
    fn event_type_try_from_roundtrip() {
        assert!(matches!(EventType::try_from("PUG"), Ok(EventType::Pug)));
        assert!(matches!(
            EventType::try_from("TOURNEY"),
            Ok(EventType::Tourney)
        ));
        assert!(EventType::try_from("OTHER").is_err());
    }

    #[test]
    fn event_format_try_from_roundtrip() {
        assert!(matches!(
            EventFormat::try_from("5v5"),
            Ok(EventFormat::FiveVFive)
        ));
        assert!(matches!(
            EventFormat::try_from("6v6"),
            Ok(EventFormat::SixVSix)
        ));
        assert!(matches!(
            EventFormat::try_from("1v1"),
            Ok(EventFormat::OneVOne)
        ));
        assert!(EventFormat::try_from("2v2").is_err());
    }
}
