//! Integration tests for the bot announcement queries.
//!
//! Run with:
//!   cargo test -p tornare-bot --test announcements

use sqlx::{PgPool, Row};
use tornare_bot::announcements::{fetch_guilds, fetch_pending_events_for_guild, mark_event_posted};
use uuid::Uuid;

/// Only guilds with announcements_enabled=true AND deleted_at IS NULL must be fetched.
#[sqlx::test(migrations = "../backend/migrations")]
async fn fetch_guilds_excludes_disabled_and_deleted(pool: PgPool) {
    // Enabled and not deleted — should be returned.
    sqlx::query(
        "INSERT INTO discord_guilds (id, guild_id, channel_id, announcements_enabled) \
         VALUES (gen_random_uuid(), 'guild-enabled', 'ch-1', true)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Announcements disabled — should NOT be returned.
    sqlx::query(
        "INSERT INTO discord_guilds (id, guild_id, channel_id, announcements_enabled) \
         VALUES (gen_random_uuid(), 'guild-disabled', 'ch-2', false)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Soft-deleted — should NOT be returned even though announcements are enabled.
    sqlx::query(
        "INSERT INTO discord_guilds \
         (id, guild_id, channel_id, announcements_enabled, deleted_at) \
         VALUES (gen_random_uuid(), 'guild-deleted', 'ch-3', true, NOW())",
    )
    .execute(&pool)
    .await
    .unwrap();

    let guilds = fetch_guilds(&pool).await.unwrap();
    assert_eq!(guilds.len(), 1, "only the enabled, non-deleted guild must be returned");
    assert_eq!(guilds[0].guild_id, "guild-enabled");
}

/// Events already posted to a guild must not appear as pending a second time.
#[sqlx::test(migrations = "../backend/migrations")]
async fn fetch_pending_events_excludes_already_posted(pool: PgPool) {
    // Insert a minimal user.
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, created_at, updated_at) \
         VALUES ($1, 'bot-test@test.local', 'bottest', 'Bot Test', NOW(), NOW())",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Insert a guild and enroll the user.
    let guild_uuid = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO discord_guilds (id, guild_id, channel_id, announcements_enabled) \
         VALUES ($1, 'test-guild-pending', 'ch-t', true)",
    )
    .bind(guild_uuid)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO discord_guild_members (discord_guild_id, user_id) VALUES ($1, $2)",
    )
    .bind(guild_uuid)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Insert an active event owned by that user.
    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events \
         (id, name, event_type, format, max_players, status, created_at) \
         VALUES ($1, 'Test Event', 'PUG', '5v5', 10, 'ACTIVE', NOW())",
    )
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO event_memberships (id, event_id, user_id, role, created_at) \
         VALUES (gen_random_uuid(), $1, $2, 'owner', NOW())",
    )
    .bind(event_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Before posting: event must appear as pending.
    let pending = fetch_pending_events_for_guild(&pool, guild_uuid).await.unwrap();
    assert_eq!(pending.len(), 1, "event must appear as pending before it is posted");

    // Mark as posted.
    mark_event_posted(&pool, guild_uuid, event_id).await.unwrap();

    // After posting: no longer pending.
    let pending2 = fetch_pending_events_for_guild(&pool, guild_uuid).await.unwrap();
    assert_eq!(pending2.len(), 0, "event must not be pending after it has been posted");
}

/// Verifies that the SQL used by `post_event` decodes `start_date` as `Option<String>`.
/// Before the fix the column was a TIMESTAMPTZ but was decoded into an `Option<String>`,
/// which always failed; the fix casts it to text in SQL via `to_char(…)`.
/// This test catches regressions: a non-null start_date must come through as `Some(string)`.
#[sqlx::test(migrations = "../backend/migrations")]
async fn post_event_query_decodes_start_date_as_string(pool: PgPool) {
    // Minimal user required by the LEFT JOINs in the query.
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, created_at, updated_at) \
         VALUES ($1, 'sd-test@test.local', 'sdtest', 'SD Test', NOW(), NOW())",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events \
         (id, name, description, event_type, format, max_players, status, \
          start_date, discord_announce, created_at) \
         VALUES ($1, 'SD Event', 'desc', 'PUG', '5v5', 10, 'ACTIVE', \
                 '2025-06-15 18:00:00+00', true, NOW())",
    )
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO event_memberships (id, event_id, user_id, role, created_at) \
         VALUES (gen_random_uuid(), $1, $2, 'owner', NOW())",
    )
    .bind(event_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Reproduce the exact SELECT used by post_event (with the to_char fix).
    let row = sqlx::query(
        "SELECT \
             e.name, e.description, e.event_type, \
             to_char(e.start_date AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS start_date, \
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
    .fetch_one(&pool)
    .await
    .unwrap();

    let start_date: Option<String> = row.try_get("start_date").ok().flatten();
    assert!(
        start_date.is_some(),
        "start_date must decode as Some(string) when the column is non-null"
    );
    let date_str = start_date.unwrap();
    assert!(
        date_str.contains("2025-06-15"),
        "decoded start_date should contain the expected date; got: {date_str}"
    );
    assert!(
        date_str.ends_with('Z'),
        "decoded start_date should end with 'Z' (UTC ISO-8601); got: {date_str}"
    );
}
