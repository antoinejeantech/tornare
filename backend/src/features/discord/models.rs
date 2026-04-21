use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DiscordGuild {
    pub id: Uuid,
    pub guild_id: String,
    pub guild_name: Option<String>,
    pub owner_user_id: Option<Uuid>,
    pub channel_id: String,
    pub announcements_enabled: bool,
    pub mention_roles: Vec<String>,
    pub last_post_error: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::features::events::domain::serialize_optional_timestamp"
    )]
    pub last_post_error_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
pub struct GuildMember {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub display_name: String,
    #[serde(serialize_with = "crate::features::events::domain::serialize_timestamp")]
    pub added_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct AddGuildMemberInput {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpsertGuildInput {
    /// Discord snowflake for the server
    pub guild_id: String,
    /// Human-readable name (optional, shown in UI)
    pub guild_name: Option<String>,
    /// Discord snowflake for the announcement channel
    pub channel_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleAnnouncementsInput {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetMentionRolesInput {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BotInviteUrl {
    pub url: String,
}
