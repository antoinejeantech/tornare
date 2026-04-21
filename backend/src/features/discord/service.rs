use std::env;

use uuid::Uuid;

use crate::{
    app::state::AppState,
    shared::errors::{bad_request, forbidden, not_found, ApiError},
};

use super::{
    models::{
        AddGuildMemberInput, BotInviteUrl, DiscordGuild, GuildMember,
        SetMentionRolesInput, ToggleAnnouncementsInput, UpsertGuildInput,
    },
    repo,
};

pub async fn get_my_guilds(state: &AppState, user_id: Uuid) -> Result<Vec<DiscordGuild>, ApiError> {
    repo::find_guilds_by_owner(&state.pool, user_id).await
}

pub async fn upsert_my_guild(
    state: &AppState,
    user_id: Uuid,
    input: UpsertGuildInput,
) -> Result<DiscordGuild, ApiError> {
    let guild_id = input.guild_id.trim();
    let channel_id = input.channel_id.trim();

    if guild_id.is_empty() {
        return Err(bad_request("guild_id is required"));
    }
    if channel_id.is_empty() {
        return Err(bad_request("channel_id is required"));
    }

    // The owner must have Discord connected to manage a Discord guild.
    let owner_has_discord: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM auth_identities WHERE provider = 'discord' AND user_id = $1)",
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(crate::shared::errors::internal_error)?;
    if !owner_has_discord {
        return Err(bad_request("You must connect your Discord account before managing a Discord guild"));
    }

    // If another owner already registered this guild, block the takeover.
    let existing: Option<Option<Uuid>> = sqlx::query_scalar(
        "SELECT owner_user_id FROM discord_guilds WHERE guild_id = $1 AND deleted_at IS NULL",
    )
    .bind(guild_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(crate::shared::errors::internal_error)?;

    if let Some(Some(existing_owner)) = existing {
        if existing_owner != user_id {
            return Err(forbidden("This Discord server is already registered by another user"));
        }
    }

    repo::upsert_guild(
        &state.pool,
        user_id,
        guild_id,
        input.guild_name.as_deref(),
        channel_id,
    )
    .await
}

pub async fn toggle_announcements(
    state: &AppState,
    user_id: Uuid,
    guild_id: &str,
    input: ToggleAnnouncementsInput,
) -> Result<DiscordGuild, ApiError> {
    let updated =
        repo::set_announcements_enabled(&state.pool, user_id, guild_id, input.enabled).await?;
    if !updated {
        return Err(not_found("No Discord guild configured"));
    }
    repo::find_guild_by_guild_id_and_owner(&state.pool, guild_id, user_id)
        .await?
        .ok_or_else(|| not_found("No Discord guild configured"))
}

pub async fn delete_my_guild(
    state: &AppState,
    user_id: Uuid,
    guild_id: &str,
) -> Result<(), ApiError> {
    let deleted = repo::delete_guild_by_owner(&state.pool, user_id, guild_id).await?;
    if !deleted {
        return Err(not_found("Guild not found or not owned by you"));
    }
    Ok(())
}

pub async fn list_guild_members(
    state: &AppState,
    user_id: Uuid,
    guild_id: &str,
) -> Result<Vec<GuildMember>, ApiError> {
    let guild = repo::find_guild_by_guild_id_and_owner(&state.pool, guild_id, user_id)
        .await?
        .ok_or_else(|| not_found("Guild not found or not owned by you"))?;
    repo::list_guild_members(&state.pool, guild.id).await
}

pub async fn add_guild_member(
    state: &AppState,
    owner_id: Uuid,
    guild_id: &str,
    input: AddGuildMemberInput,
) -> Result<Vec<GuildMember>, ApiError> {
    let guild = repo::find_guild_by_guild_id_and_owner(&state.pool, guild_id, owner_id)
        .await?
        .ok_or_else(|| not_found("Guild not found or not owned by you"))?;

    // Verify the target user has Discord connected.
    let has_discord: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM auth_identities WHERE provider = 'discord' AND user_id = $1)",
    )
    .bind(input.user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(crate::shared::errors::internal_error)?;
    if !has_discord {
        return Err(bad_request("This user must connect their Discord account before being added to a guild"));
    }

    repo::add_guild_member(&state.pool, guild.id, input.user_id).await?;
    repo::list_guild_members(&state.pool, guild.id).await
}

pub async fn remove_guild_member(
    state: &AppState,
    owner_id: Uuid,
    guild_id: &str,
    target_user_id: Uuid,
) -> Result<Vec<GuildMember>, ApiError> {
    let guild = repo::find_guild_by_guild_id_and_owner(&state.pool, guild_id, owner_id)
        .await?
        .ok_or_else(|| not_found("Guild not found or not owned by you"))?;

    let owner_uid = guild
        .owner_user_id
        .ok_or_else(|| not_found("Guild has no owner"))?;

    if target_user_id == owner_uid {
        return Err(bad_request("Cannot remove the guild owner from the pool"));
    }

    repo::remove_guild_member(&state.pool, guild.id, target_user_id, owner_uid).await?;
    repo::list_guild_members(&state.pool, guild.id).await
}

pub async fn set_mention_roles(
    state: &AppState,
    user_id: Uuid,
    guild_id: &str,
    input: SetMentionRolesInput,
) -> Result<DiscordGuild, ApiError> {
    // Validate: every role ID must be a Discord snowflake (17-20 decimal digits).
    for role_id in &input.roles {
        if !role_id.chars().all(|c| c.is_ascii_digit())
            || role_id.len() < 17
            || role_id.len() > 20
        {
            return Err(bad_request("INVALID_ROLE_ID"));
        }
    }

    repo::set_mention_roles(&state.pool, user_id, guild_id, &input.roles)
        .await?
        .ok_or_else(|| not_found("No Discord guild configured"))
}

pub fn get_bot_invite_url(state: &AppState) -> BotInviteUrl {
    let client_id = env::var("DISCORD_BOT_CLIENT_ID")
        .unwrap_or_else(|_| state.config.discord_client_id.clone());

    // Permissions: Send Messages (2048) + Embed Links (16384) + View Channel (1024) = 19456
    // Scopes: bot + applications.commands (needed for slash commands)
    let url = format!(
        "https://discord.com/api/oauth2/authorize\
         ?client_id={client_id}\
         &permissions=19456\
         &scope=bot%20applications.commands"
    );
    BotInviteUrl { url }
}
