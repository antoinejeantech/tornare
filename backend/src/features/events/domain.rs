use serde::{Deserialize, Serialize, Serializer};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Timestamp serialization helpers
// ---------------------------------------------------------------------------

pub(super) fn serialize_timestamp<S>(value: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = value
        .to_offset(UtcOffset::UTC)
        .format(&Rfc3339)
        .map_err(serde::ser::Error::custom)?;

    serializer.serialize_str(&formatted)
}

pub(super) fn serialize_optional_timestamp<S>(
    value: &Option<OffsetDateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serialize_timestamp(value, serializer),
        None => serializer.serialize_none(),
    }
}

// ---------------------------------------------------------------------------
// Event type / format
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Bracket generation mode
// ---------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BracketGenerationMode {
    Random,
    Empty,
}

// ---------------------------------------------------------------------------
// Player role
// ---------------------------------------------------------------------------

/// The role a player fills in a match. Serialized as "Tank", "DPS", "Support".
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerRole {
    Tank,
    #[serde(rename = "DPS")]
    Dps,
    Support,
}

impl PlayerRole {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            PlayerRole::Tank => "Tank",
            PlayerRole::Dps => "DPS",
            PlayerRole::Support => "Support",
        }
    }
}

impl TryFrom<&str> for PlayerRole {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Tank" => Ok(PlayerRole::Tank),
            "DPS" => Ok(PlayerRole::Dps),
            "Support" => Ok(PlayerRole::Support),
            other => Err(format!("Invalid player role: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Player rank
// ---------------------------------------------------------------------------

/// Overwatch competitive rank. Serialized as the rank name string.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PlayerRank {
    Unranked,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Master,
    Grandmaster,
    Champion,
}

impl PlayerRank {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            PlayerRank::Unranked => "Unranked",
            PlayerRank::Bronze => "Bronze",
            PlayerRank::Silver => "Silver",
            PlayerRank::Gold => "Gold",
            PlayerRank::Platinum => "Platinum",
            PlayerRank::Diamond => "Diamond",
            PlayerRank::Master => "Master",
            PlayerRank::Grandmaster => "Grandmaster",
            PlayerRank::Champion => "Champion",
        }
    }
}

impl TryFrom<&str> for PlayerRank {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Unranked" => Ok(PlayerRank::Unranked),
            "Bronze" => Ok(PlayerRank::Bronze),
            "Silver" => Ok(PlayerRank::Silver),
            "Gold" => Ok(PlayerRank::Gold),
            "Platinum" => Ok(PlayerRank::Platinum),
            "Diamond" => Ok(PlayerRank::Diamond),
            "Master" => Ok(PlayerRank::Master),
            "Grandmaster" => Ok(PlayerRank::Grandmaster),
            "Champion" => Ok(PlayerRank::Champion),
            other => Err(format!("Invalid player rank: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Role preference (used in signup requests)
// ---------------------------------------------------------------------------

/// A single role+rank preference submitted by an applicant.
#[derive(Serialize, Clone)]
pub struct RolePreference {
    pub role: PlayerRole,
    pub rank: PlayerRank,
}

// ---------------------------------------------------------------------------
// Signup request status
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SignupStatus {
    Pending,
    Accepted,
    Declined,
}

impl SignupStatus {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            SignupStatus::Pending => "pending",
            SignupStatus::Accepted => "accepted",
            SignupStatus::Declined => "declined",
        }
    }
}

impl TryFrom<&str> for SignupStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(SignupStatus::Pending),
            "accepted" => Ok(SignupStatus::Accepted),
            "declined" => Ok(SignupStatus::Declined),
            other => Err(format!("Invalid signup status: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Match status
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum MatchStatus {
    Open,
    Ready,
    Completed,
}

impl MatchStatus {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            MatchStatus::Open => "OPEN",
            MatchStatus::Ready => "READY",
            MatchStatus::Completed => "COMPLETED",
        }
    }
}

impl TryFrom<&str> for MatchStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "OPEN" => Ok(MatchStatus::Open),
            "READY" => Ok(MatchStatus::Ready),
            "COMPLETED" => Ok(MatchStatus::Completed),
            other => Err(format!("Invalid match status: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Domain aggregates
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub role: PlayerRole,
    pub rank: PlayerRank,
    pub team_id: Option<Uuid>,
    pub team: Option<String>,
    /// Role this player is assigned to play in their current team.
    /// Set by auto-balance or manual assignment with role; `None` when the
    /// player has no team or was moved without an explicit role choice.
    pub assigned_role: Option<PlayerRole>,
    pub assigned_rank: Option<PlayerRank>,
    /// Role preferences for this player, always populated.
    /// For manually-added players these are owner-set; for accepted signup
    /// requests they are copied from the original application.
    pub roles: Vec<RolePreference>,
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
    pub status: MatchStatus,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: OffsetDateTime,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: OffsetDateTime,
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub start_date: Option<OffsetDateTime>,
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
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub start_date: Option<OffsetDateTime>,
    pub event_type: EventType,
    pub format: EventFormat,
    pub is_featured: bool,
    pub is_ended: bool,
    pub is_owner: bool,
    pub can_manage: bool,
    pub creator_id: Option<Uuid>,
    pub creator_name: Option<String>,
    pub public_signup_enabled: bool,
    pub public_signup_token: Option<String>,
    pub max_players: u8,
    pub players: Vec<Player>,
    pub teams: Vec<EventTeam>,
    pub matches: Vec<Match>,
}

impl Event {
    /// Set ownership/management flags for a write-path response.
    /// `is_owner` is `true` for literal event-membership owners, `false` for
    /// global admins/moderators acting with elevated access.
    pub fn into_owner(mut self, is_owner: bool) -> Self {
        self.is_owner = is_owner;
        self.can_manage = true;
        self
    }
}

#[derive(Serialize, Clone)]
pub struct EventSignupRequest {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub roles: Vec<RolePreference>,
    pub status: SignupStatus,
}

#[derive(Serialize)]
pub struct PublicEventSignupInfo {
    pub event_id: Uuid,
    pub event_name: String,
    pub event_description: String,
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub start_date: Option<OffsetDateTime>,
    pub event_type: EventType,
    pub format: EventFormat,
    pub max_players: u8,
    pub current_players: usize,
    pub current_signup_requests: usize,
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn player_role_round_trip() {
        for (s, expected) in [
            ("Tank", PlayerRole::Tank),
            ("DPS", PlayerRole::Dps),
            ("Support", PlayerRole::Support),
        ] {
            let role = PlayerRole::try_from(s).unwrap();
            assert_eq!(role, expected);
            assert_eq!(role.as_db_value(), s);
        }
    }

    #[test]
    fn player_rank_round_trip() {
        for s in ["Unranked", "Bronze", "Silver", "Gold", "Platinum", "Diamond", "Master", "Grandmaster", "Champion"] {
            let rank = PlayerRank::try_from(s).unwrap();
            assert_eq!(rank.as_db_value(), s);
        }
    }

    #[test]
    fn signup_status_round_trip() {
        for (s, expected) in [
            ("pending", SignupStatus::Pending),
            ("accepted", SignupStatus::Accepted),
            ("declined", SignupStatus::Declined),
        ] {
            let status = SignupStatus::try_from(s).unwrap();
            assert_eq!(status, expected);
            assert_eq!(status.as_db_value(), s);
        }
    }

    #[test]
    fn match_status_round_trip() {
        for (s, expected) in [
            ("OPEN", MatchStatus::Open),
            ("READY", MatchStatus::Ready),
            ("COMPLETED", MatchStatus::Completed),
        ] {
            let status = MatchStatus::try_from(s).unwrap();
            assert_eq!(status, expected);
            assert_eq!(status.as_db_value(), s);
        }
    }
}
