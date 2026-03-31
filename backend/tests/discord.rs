//! Tests for Discord OAuth features: disconnect guard and hard-delete disconnect behavior.
//!
//! Run with:
//!   cargo test --test discord

mod common;

use common::{register, spawn_test_server};
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Discord disconnect guard
// ---------------------------------------------------------------------------

/// A user whose only login method is Discord (no password) must receive a
/// 400 if they try to disconnect, so they cannot lock themselves out.
#[sqlx::test]
async fn disconnect_discord_is_blocked_when_user_has_no_password(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body = register(&client, &base, "discordonly@test.local", "discordonly").await;
    let token = body["access_token"].as_str().expect("must have token").to_string();
    let user_id = Uuid::parse_str(
        body["user"]["id"].as_str().expect("must have user.id"),
    )
    .expect("user.id must be a valid UUID");

    // Simulate a discord-only user: insert a discord identity and remove the password.
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id)
         VALUES (gen_random_uuid(), $1, 'discord', 'test-discord-sub-no-pwd')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to insert discord identity");

    sqlx::query("UPDATE users SET password_hash = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("failed to clear password_hash");

    let res = client
        .delete(format!("{base}/api/auth/discord/disconnect"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("disconnect request failed");

    assert_eq!(
        res.status().as_u16(),
        400,
        "disconnect must be blocked for a user with no password (lockout prevention)"
    );
}

/// A user who has a password set can disconnect Discord without being blocked.
#[sqlx::test]
async fn disconnect_discord_succeeds_when_user_has_password(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body = register(&client, &base, "discordpwd@test.local", "discordpwd").await;
    let token = body["access_token"].as_str().expect("must have token").to_string();
    let user_id = Uuid::parse_str(
        body["user"]["id"].as_str().expect("must have user.id"),
    )
    .expect("user.id must be a valid UUID");

    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id)
         VALUES (gen_random_uuid(), $1, 'discord', 'test-discord-sub-with-pwd')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to insert discord identity");

    let res = client
        .delete(format!("{base}/api/auth/discord/disconnect"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("disconnect request failed");

    assert_eq!(
        res.status().as_u16(),
        200,
        "disconnect must succeed when the user has a password set"
    );
}

/// After a successful disconnect the identity row must be hard-deleted —
/// no ghost row should remain under any provider name.
#[sqlx::test]
async fn disconnect_discord_hard_deletes_identity(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body = register(&client, &base, "discordsoft@test.local", "discordsoft").await;
    let token = body["access_token"].as_str().expect("must have token").to_string();
    let user_id = Uuid::parse_str(
        body["user"]["id"].as_str().expect("must have user.id"),
    )
    .expect("user.id must be a valid UUID");

    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id)
         VALUES (gen_random_uuid(), $1, 'discord', 'test-discord-sub-soft-del')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to insert discord identity");

    let res = client
        .delete(format!("{base}/api/auth/discord/disconnect"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("disconnect request failed");
    assert_eq!(res.status().as_u16(), 200, "disconnect must succeed");

    // The identity must be fully removed — no row under any provider.
    let row_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM auth_identities
         WHERE user_id = $1 AND provider_user_id = 'test-discord-sub-soft-del'",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("DB query failed");

    assert_eq!(
        row_count.0, 0,
        "identity row must be fully deleted after disconnect, no ghost rows"
    );
}
