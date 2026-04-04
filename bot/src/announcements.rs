use sqlx::{PgPool, Row};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::discord::{DiscordHttp, EventEmbed};

/// A guild ready to receive announcements.
pub struct Guild {
    pub id: Uuid,
    pub guild_id: String,
    pub channel_id: String,
}

/// Load all guilds with announcements enabled.
pub async fn fetch_guilds(pool: &PgPool) -> anyhow::Result<Vec<Guild>> {
    let rows = sqlx::query(
        "SELECT id, guild_id, channel_id \
         FROM discord_guilds \
         WHERE announcements_enabled = TRUE AND deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| Guild {
            id: r.get("id"),
            guild_id: r.get("guild_id"),
            channel_id: r.get("channel_id"),
        })
        .collect())
}

/// Backfill guild names for rows where guild_name is NULL (set via /setup).
pub async fn backfill_guild_names(pool: &PgPool, http: &DiscordHttp) {
    let rows = match sqlx::query(
        "SELECT guild_id FROM discord_guilds WHERE guild_name IS NULL",
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("Failed to query guilds missing names: {e:#}");
            return;
        }
    };

    for row in rows {
        let guild_id: String = row.get("guild_id");
        match http.fetch_guild_name(&guild_id).await {
            Ok(Some(name)) => {
                if let Err(e) = sqlx::query(
                    "UPDATE discord_guilds SET guild_name = $1, updated_at = NOW() \
                     WHERE guild_id = $2",
                )
                .bind(&name)
                .bind(&guild_id)
                .execute(pool)
                .await
                {
                    warn!("Failed to update guild name for {guild_id}: {e:#}");
                } else {
                    info!("Backfilled guild name for {guild_id}: {name}");
                }
            }
            Ok(None) => {}
            Err(e) => warn!("Failed to fetch guild name for {guild_id}: {e:#}"),
        }
    }
}

/// Returns IDs of active events that belong to guild members and have not yet
/// been posted to this specific guild.
pub async fn fetch_pending_events_for_guild(
    pool: &PgPool,
    discord_guild_id: Uuid,
) -> anyhow::Result<Vec<Uuid>> {
    let rows = sqlx::query(
        "SELECT DISTINCT e.id \
         FROM events e \
         JOIN event_memberships em ON em.event_id = e.id AND em.role = 'owner' \
         JOIN discord_guild_members dgm \
             ON dgm.user_id = em.user_id AND dgm.discord_guild_id = $1 \
         WHERE e.status = 'ACTIVE' \
           AND e.deleted_at IS NULL \
           AND NOT EXISTS ( \
               SELECT 1 FROM discord_guild_posts dgp \
               WHERE dgp.discord_guild_id = $1 AND dgp.event_id = e.id \
           )",
    )
    .bind(discord_guild_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| r.get("id")).collect())
}

/// Record that an event was successfully posted to a guild.
pub async fn mark_event_posted(
    pool: &PgPool,
    discord_guild_id: Uuid,
    event_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO discord_guild_posts (discord_guild_id, event_id) \
         VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(discord_guild_id)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetch event data and post the embed to the guild's channel.
pub async fn post_event(
    pool: &PgPool,
    http: &DiscordHttp,
    channel_id: &str,
    frontend_url: &str,
    event_id: Uuid,
) -> anyhow::Result<()> {
    let row = sqlx::query(
        "SELECT \
             e.name, e.description, e.event_type, e.start_date, \
             e.format, e.max_players, \
             e.public_signup_enabled, e.signup_token, \
             u.display_name AS organizer_name \
         FROM events e \
         LEFT JOIN event_memberships em ON em.event_id = e.id AND em.role = 'owner' \
         LEFT JOIN users u ON u.id = em.user_id \
         WHERE e.id = $1 \
         LIMIT 1",
    )
    .bind(event_id)
    .fetch_one(pool)
    .await?;

    let event_url = format!("{frontend_url}/events/{event_id}");

    let public_signup_enabled: bool = row.try_get("public_signup_enabled").unwrap_or(false);
    let signup_token: Option<String> = row.try_get("signup_token").ok().flatten();
    let join_url = if !public_signup_enabled {
        signup_token.map(|t| format!("{frontend_url}/join/{t}"))
    } else {
        None
    };

    let embed = EventEmbed {
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        event_type: row.try_get("event_type")?,
        format: row.try_get("format")?,
        max_players: row.try_get("max_players")?,
        start_date: row.try_get("start_date").ok().flatten(),
        organizer: row.try_get("organizer_name").ok().flatten(),
        join_url,
        event_url,
    };

    let message_id = http.post_event_embed(channel_id, &embed).await?;
    info!(
        "Posted Discord message {message_id} for event '{}' ({event_id}) -> channel {channel_id}",
        embed.name
    );
    Ok(())
}

/// Run one poll cycle: for each guild, find and post pending events.
pub async fn run_poll(pool: &PgPool, http: &DiscordHttp, frontend_url: &str) {
    info!("Poll started");
    backfill_guild_names(pool, http).await;

    let guilds = match fetch_guilds(pool).await {
        Ok(g) => g,
        Err(e) => {
            error!("Failed to fetch Discord guilds: {e:#}");
            return;
        }
    };

    info!("Checking {} guild(s) for pending events", guilds.len());

    for guild in &guilds {
        let pending = match fetch_pending_events_for_guild(pool, guild.id).await {
            Ok(ids) => ids,
            Err(e) => {
                error!(
                    "Failed to fetch pending events for guild {}: {e:#}",
                    guild.guild_id
                );
                continue;
            }
        };

        info!(
            "Guild {} — {} pending event(s)",
            guild.guild_id,
            pending.len()
        );

        let had_pending = !pending.is_empty();
        let mut last_error: Option<String> = None;

        for event_id in pending {
            match post_event(pool, http, &guild.channel_id, frontend_url, event_id).await {
                Ok(()) => {
                    if let Err(e) = mark_event_posted(pool, guild.id, event_id).await {
                        error!(
                            "Failed to record post for event {event_id} in guild {}: {e:#}",
                            guild.guild_id
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to post event {event_id} to channel {}: {e:#}",
                        guild.channel_id
                    );
                    last_error = Some(e.to_string());
                }
            }
        }

        if let Some(ref err) = last_error {
            match http.is_bot_in_guild(&guild.guild_id).await {
                Ok(false) => {
                    warn!(
                        "Bot is no longer in guild {} — disabling announcements",
                        guild.guild_id
                    );
                    if let Err(e) = sqlx::query(
                        "UPDATE discord_guilds \
                         SET announcements_enabled = FALSE, \
                             last_post_error = $2, last_post_error_at = NOW(), \
                             updated_at = NOW() \
                         WHERE id = $1",
                    )
                    .bind(guild.id)
                    .bind("Bot was removed from the server")
                    .execute(pool)
                    .await
                    {
                        error!("Failed to disable guild {}: {e:#}", guild.guild_id);
                    }
                }
                Ok(true) => {
                    // Bot still present — record the error for the frontend.
                    if let Err(e) = sqlx::query(
                        "UPDATE discord_guilds \
                         SET last_post_error = $2, last_post_error_at = NOW(), updated_at = NOW() \
                         WHERE id = $1",
                    )
                    .bind(guild.id)
                    .bind(err)
                    .execute(pool)
                    .await
                    {
                        error!("Failed to record post error for guild {}: {e:#}", guild.guild_id);
                    }
                }
                Err(e) => {
                    warn!("Could not verify guild membership for {}: {e:#}", guild.guild_id);
                }
            }
        } else if had_pending {
            // All events posted successfully — clear any previous error.
            if let Err(e) = sqlx::query(
                "UPDATE discord_guilds \
                 SET last_post_error = NULL, last_post_error_at = NULL, updated_at = NOW() \
                 WHERE id = $1 AND last_post_error IS NOT NULL",
            )
            .bind(guild.id)
            .execute(pool)
            .await
            {
                error!("Failed to clear post error for guild {}: {e:#}", guild.guild_id);
            }
        }
    }
}
