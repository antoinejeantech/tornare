use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::shared::errors::{internal_error, ApiError};

use super::models::{DiscordGuild, GuildMember};

fn row_to_guild(r: &sqlx::postgres::PgRow) -> DiscordGuild {
    DiscordGuild {
        id: r.get("id"),
        guild_id: r.get("guild_id"),
        guild_name: r.get("guild_name"),
        owner_user_id: r.get("owner_user_id"),
        channel_id: r.get("channel_id"),
        announcements_enabled: r.get("announcements_enabled"),
        last_post_error: r.get("last_post_error"),
        last_post_error_at: r.get("last_post_error_at"),
    }
}

/// Returns all guilds owned by the given user.
pub async fn find_guilds_by_owner(
    pool: &PgPool,
    owner_user_id: Uuid,
) -> Result<Vec<DiscordGuild>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, guild_id, guild_name, owner_user_id, channel_id, announcements_enabled, last_post_error, last_post_error_at \
         FROM discord_guilds \
         WHERE owner_user_id = $1 AND deleted_at IS NULL \
         ORDER BY created_at",
    )
    .bind(owner_user_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows.iter().map(row_to_guild).collect())
}

/// Returns the guild matching the Discord snowflake guild_id, owned by the given user.
pub async fn find_guild_by_guild_id_and_owner(
    pool: &PgPool,
    guild_id: &str,
    owner_user_id: Uuid,
) -> Result<Option<DiscordGuild>, ApiError> {
    let row = sqlx::query(
        "SELECT id, guild_id, guild_name, owner_user_id, channel_id, announcements_enabled, last_post_error, last_post_error_at \
         FROM discord_guilds \
         WHERE guild_id = $1 AND owner_user_id = $2 AND deleted_at IS NULL \
         LIMIT 1",
    )
    .bind(guild_id)
    .bind(owner_user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.as_ref().map(row_to_guild))
}

#[allow(dead_code)]
pub async fn find_guild_by_guild_id(
    pool: &PgPool,
    guild_id: &str,
) -> Result<Option<DiscordGuild>, ApiError> {
    let row = sqlx::query(
        "SELECT id, guild_id, guild_name, owner_user_id, channel_id, announcements_enabled, last_post_error, last_post_error_at \
         FROM discord_guilds \
         WHERE guild_id = $1 AND deleted_at IS NULL \
         LIMIT 1",
    )
    .bind(guild_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.as_ref().map(row_to_guild))
}

pub async fn upsert_guild(
    pool: &PgPool,
    owner_user_id: Uuid,
    guild_id: &str,
    guild_name: Option<&str>,
    channel_id: &str,
) -> Result<DiscordGuild, ApiError> {
    // Detect a soft-deleted guild being reclaimed so we can clear the stale
    // member pool after the upsert (avoid cross-account data leak).
    let is_reclaim: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM discord_guilds WHERE guild_id = $1 AND deleted_at IS NOT NULL)",
    )
    .bind(guild_id)
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    let row = sqlx::query(
        "INSERT INTO discord_guilds (id, guild_id, guild_name, owner_user_id, channel_id)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)
         ON CONFLICT (guild_id) DO UPDATE
             SET guild_name    = EXCLUDED.guild_name,
                 owner_user_id = EXCLUDED.owner_user_id,
                 channel_id    = EXCLUDED.channel_id,
                 deleted_at    = NULL,
                 updated_at    = NOW()
         RETURNING id, guild_id, guild_name, owner_user_id, channel_id, announcements_enabled, last_post_error, last_post_error_at",
    )
    .bind(guild_id)
    .bind(guild_name)
    .bind(owner_user_id)
    .bind(channel_id)
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    let guild = row_to_guild(&row);

    if is_reclaim {
        // Purge the previous owner's announcement pool so the new owner starts
        // fresh. discord_guild_posts is intentionally left intact to prevent
        // re-announcing events that were already posted.
        sqlx::query("DELETE FROM discord_guild_members WHERE discord_guild_id = $1")
            .bind(guild.id)
            .execute(pool)
            .await
            .map_err(internal_error)?;
    }

    // Ensure the owner is in the member pool.
    sqlx::query(
        "INSERT INTO discord_guild_members (discord_guild_id, user_id) \
         VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(guild.id)
    .bind(owner_user_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(guild)
}

/// Called by the /setup slash command — no known owner yet.
pub async fn upsert_guild_from_slash(
    pool: &PgPool,
    guild_id: &str,
    guild_name: Option<&str>,
    channel_id: &str,
    discord_user_id: &str,
) -> Result<DiscordGuild, ApiError> {
    // Try to find a matching user by Discord identity so we can set owner_user_id.
    let owner_user_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM auth_identities WHERE provider = 'discord' AND provider_user_id = $1 LIMIT 1",
    )
    .bind(discord_user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    let row = sqlx::query(
        "INSERT INTO discord_guilds (id, guild_id, guild_name, owner_user_id, channel_id)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)
         ON CONFLICT (guild_id) DO UPDATE
             SET guild_name    = EXCLUDED.guild_name,
                 -- Only allow taking ownership if the existing row is soft-deleted.
                 -- An active guild's owner can never be changed via /setup.
                 owner_user_id = CASE
                     WHEN discord_guilds.deleted_at IS NOT NULL THEN EXCLUDED.owner_user_id
                     ELSE COALESCE(discord_guilds.owner_user_id, EXCLUDED.owner_user_id)
                     END,
                 channel_id    = EXCLUDED.channel_id,
                 deleted_at    = NULL,
                 updated_at    = NOW()
         RETURNING id, guild_id, guild_name, owner_user_id, channel_id, announcements_enabled, last_post_error, last_post_error_at",
    )
    .bind(guild_id)
    .bind(guild_name)
    .bind(owner_user_id)
    .bind(channel_id)
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    let guild = row_to_guild(&row);

    // On reclaim, purge any members that belonged to the previous owner so the
    // new owner starts with a clean announcement pool. Deleting rows where
    // user_id != current_owner is a no-op on a normal (non-reclaim) upsert.
    // discord_guild_posts is left intact to avoid re-announcing old events.
    if let Some(uid) = guild.owner_user_id {
        sqlx::query(
            "DELETE FROM discord_guild_members
             WHERE discord_guild_id = $1 AND user_id != $2",
        )
        .bind(guild.id)
        .bind(uid)
        .execute(pool)
        .await
        .map_err(internal_error)?;

        // Ensure the owner is in the member pool.
        sqlx::query(
            "INSERT INTO discord_guild_members (discord_guild_id, user_id) \
             VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(guild.id)
        .bind(uid)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    }

    Ok(guild)
}

pub async fn set_announcements_enabled(
    pool: &PgPool,
    owner_user_id: Uuid,
    guild_id: &str,
    enabled: bool,
) -> Result<bool, ApiError> {
    let result = sqlx::query(
        "UPDATE discord_guilds SET announcements_enabled = $1, updated_at = NOW() \
         WHERE owner_user_id = $2 AND guild_id = $3 AND deleted_at IS NULL",
    )
    .bind(enabled)
    .bind(owner_user_id)
    .bind(guild_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_guild_by_owner(
    pool: &PgPool,
    owner_user_id: Uuid,
    guild_id: &str,
) -> Result<bool, ApiError> {
    let result = sqlx::query(
        "UPDATE discord_guilds \
         SET deleted_at = NOW(), announcements_enabled = FALSE, updated_at = NOW() \
         WHERE owner_user_id = $1 AND guild_id = $2 AND deleted_at IS NULL",
    )
    .bind(owner_user_id)
    .bind(guild_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(result.rows_affected() > 0)
}

/// List all members in a guild's announcement pool.
pub async fn list_guild_members(
    pool: &PgPool,
    discord_guild_id: Uuid,
) -> Result<Vec<GuildMember>, ApiError> {
    let rows = sqlx::query(
        "SELECT dgm.user_id, u.username, u.display_name, dgm.added_at \
         FROM discord_guild_members dgm \
         JOIN users u ON u.id = dgm.user_id \
         WHERE dgm.discord_guild_id = $1 \
         ORDER BY dgm.added_at",
    )
    .bind(discord_guild_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows
        .iter()
        .map(|r| GuildMember {
            user_id: r.get("user_id"),
            username: r.get("username"),
            display_name: r.get("display_name"),
            added_at: r.get::<OffsetDateTime, _>("added_at"),
        })
        .collect())
}

/// Add a user to the guild's announcement pool. Returns false if the user doesn't exist.
pub async fn add_guild_member(
    pool: &PgPool,
    discord_guild_id: Uuid,
    user_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query(
        "INSERT INTO discord_guild_members (discord_guild_id, user_id) \
         VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(discord_guild_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(result.rows_affected() > 0)
}

/// Remove a user from the guild's announcement pool. Cannot remove the guild owner.
pub async fn remove_guild_member(
    pool: &PgPool,
    discord_guild_id: Uuid,
    user_id: Uuid,
    owner_user_id: Uuid,
) -> Result<bool, ApiError> {
    if user_id == owner_user_id {
        return Ok(false); // caller should return an error
    }
    let result = sqlx::query(
        "DELETE FROM discord_guild_members \
         WHERE discord_guild_id = $1 AND user_id = $2",
    )
    .bind(discord_guild_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(result.rows_affected() > 0)
}

/// Used by the bot via direct DB access — not called from the backend binary.
#[allow(dead_code)]
pub async fn list_all_guilds(pool: &PgPool) -> Result<Vec<DiscordGuild>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, guild_id, guild_name, owner_user_id, channel_id, announcements_enabled, last_post_error, last_post_error_at \
         FROM discord_guilds",
    )
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows.iter().map(row_to_guild).collect())
}

