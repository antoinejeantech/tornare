use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const OVERWATCH_RANKS: [&str; 9] = [
    "Unranked",
    "Bronze",
    "Silver",
    "Gold",
    "Platinum",
    "Diamond",
    "Master",
    "Grandmaster",
    "Champion",
];

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: AuthUser,
}

#[derive(Deserialize)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshInput {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutInput {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum EventType {
    Pug,
    Tourney,
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
    pub is_owner: bool,
    pub creator_name: Option<String>,
    pub max_players: u8,
    pub players: Vec<Player>,
    pub teams: Vec<EventTeam>,
    pub matches: Vec<Match>,
}

#[derive(Deserialize)]
pub struct CreateEventInput {
    pub name: String,
    pub description: String,
    pub start_date: Option<String>,
    pub event_type: EventType,
    pub max_players: u8,
}

#[derive(Deserialize)]
pub struct UpdateEventInput {
    pub name: String,
    pub description: String,
    pub start_date: Option<String>,
    pub event_type: EventType,
    pub max_players: u8,
}

#[derive(Deserialize)]
pub struct CreateEventMatchInput {
    pub title: String,
    pub map: String,
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

#[derive(Deserialize)]
pub struct CreateMatchInput {
    pub title: String,
    pub map: String,
    pub max_players: u8,
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
pub struct PublicEventSignupInfo {
    pub event_id: Uuid,
    pub event_name: String,
    pub event_type: EventType,
    pub max_players: u8,
    pub current_players: usize,
}

#[cfg(test)]
mod tests {
    use super::EventType;

    #[test]
    fn event_type_try_from_roundtrip() {
        assert!(matches!(EventType::try_from("PUG"), Ok(EventType::Pug)));
        assert!(matches!(
            EventType::try_from("TOURNEY"),
            Ok(EventType::Tourney)
        ));
        assert!(EventType::try_from("OTHER").is_err());
    }
}
