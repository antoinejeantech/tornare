//! Tests for Discord OAuth features and the Discord bot guild API.
//!
//! Run with:
//!   cargo test --test discord

mod common;

use common::{default_test_config, register, spawn_test_server, spawn_test_server_with_config};
use tornare::app::state::AppConfig;
use ed25519_dalek::{Signer, SigningKey};
use reqwest::Client;
use serde_json::{json, Value};
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

// ---------------------------------------------------------------------------
// Discord bot guild REST API
// ---------------------------------------------------------------------------

/// GET /api/discord/guilds without an auth token must return 401.
#[sqlx::test]
async fn guild_listing_requires_auth(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();
    let res = client
        .get(format!("{base}/api/discord/guilds"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 401, "unauthenticated guild listing must return 401");
}

/// PUT /api/discord/guild without an auth token must return 401.
#[sqlx::test]
async fn guild_upsert_requires_auth(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();
    let res = client
        .put(format!("{base}/api/discord/guild"))
        .json(&json!({"guild_id": "111", "channel_id": "222"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 401, "unauthenticated guild upsert must return 401");
}

/// A user who has not linked a Discord account cannot register a guild.
#[sqlx::test]
async fn guild_upsert_requires_discord_identity(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();
    let body = register(&client, &base, "nodiscord@test.local", "nodiscord").await;
    let token = body["access_token"].as_str().unwrap().to_string();

    let res = client
        .put(format!("{base}/api/discord/guild"))
        .bearer_auth(&token)
        .json(&json!({"guild_id": "9001", "channel_id": "9002"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 400, "user without discord identity must get 400");
}

/// A user with a Discord identity linked can register a guild and see it in the listing.
#[sqlx::test]
async fn guild_upsert_succeeds_for_discord_user(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();
    let body = register(&client, &base, "hasdiscord@test.local", "hasdiscord").await;
    let token = body["access_token"].as_str().unwrap().to_string();
    let user_id = uuid::Uuid::parse_str(body["user"]["id"].as_str().unwrap()).unwrap();

    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id) \
         VALUES (gen_random_uuid(), $1, 'discord', 'discord-user-100')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let res = client
        .put(format!("{base}/api/discord/guild"))
        .bearer_auth(&token)
        .json(&json!({"guild_id": "111222333", "channel_id": "444555666"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "discord user can register guild");

    let list: Value = client
        .get(format!("{base}/api/discord/guilds"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["guild_id"].as_str().unwrap(), "111222333");
}

/// A second user cannot take over a guild already registered by another user.
#[sqlx::test]
async fn guild_takeover_by_other_user_is_rejected(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body_a = register(&client, &base, "owner_a@test.local", "owner_a").await;
    let token_a = body_a["access_token"].as_str().unwrap().to_string();
    let user_a = uuid::Uuid::parse_str(body_a["user"]["id"].as_str().unwrap()).unwrap();
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id) \
         VALUES (gen_random_uuid(), $1, 'discord', 'discord-user-A')",
    )
    .bind(user_a)
    .execute(&pool)
    .await
    .unwrap();

    let res = client
        .put(format!("{base}/api/discord/guild"))
        .bearer_auth(&token_a)
        .json(&json!({"guild_id": "CONTESTED", "channel_id": "ch-a"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "first registration must succeed");

    let body_b = register(&client, &base, "owner_b@test.local", "owner_b").await;
    let token_b = body_b["access_token"].as_str().unwrap().to_string();
    let user_b = uuid::Uuid::parse_str(body_b["user"]["id"].as_str().unwrap()).unwrap();
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id) \
         VALUES (gen_random_uuid(), $1, 'discord', 'discord-user-B')",
    )
    .bind(user_b)
    .execute(&pool)
    .await
    .unwrap();

    let res2 = client
        .put(format!("{base}/api/discord/guild"))
        .bearer_auth(&token_b)
        .json(&json!({"guild_id": "CONTESTED", "channel_id": "ch-b"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status().as_u16(), 403, "guild takeover by another user must return 403");
}

/// After DELETE the guild is no longer returned by the listing endpoint.
#[sqlx::test]
async fn guild_soft_delete_hides_guild_from_listing(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();
    let body = register(&client, &base, "delowner@test.local", "delowner").await;
    let token = body["access_token"].as_str().unwrap().to_string();
    let user_id = uuid::Uuid::parse_str(body["user"]["id"].as_str().unwrap()).unwrap();

    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id) \
         VALUES (gen_random_uuid(), $1, 'discord', 'discord-del-user')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    client
        .put(format!("{base}/api/discord/guild"))
        .bearer_auth(&token)
        .json(&json!({"guild_id": "DELGUILD", "channel_id": "DELCH"}))
        .send()
        .await
        .unwrap();

    let del_res = client
        .delete(format!("{base}/api/discord/guild/DELGUILD"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(del_res.status().as_u16(), 200, "soft-delete must return 200");

    let list: Value = client
        .get(format!("{base}/api/discord/guilds"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        list.as_array().unwrap().len(),
        0,
        "soft-deleted guild must not appear in listing"
    );
}

/// After a soft-delete, a different user can re-register the same guild_id.
#[sqlx::test]
async fn soft_deleted_guild_can_be_reclaimed_by_new_owner(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    // First owner registers then removes.
    let body_a = register(&client, &base, "reclaim_a@test.local", "reclaim_a").await;
    let token_a = body_a["access_token"].as_str().unwrap().to_string();
    let user_a = uuid::Uuid::parse_str(body_a["user"]["id"].as_str().unwrap()).unwrap();
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id) \
         VALUES (gen_random_uuid(), $1, 'discord', 'discord-reclaim-A')",
    )
    .bind(user_a)
    .execute(&pool)
    .await
    .unwrap();
    client
        .put(format!("{base}/api/discord/guild"))
        .bearer_auth(&token_a)
        .json(&json!({"guild_id": "RECLAIMGUILD", "channel_id": "ch-r"}))
        .send()
        .await
        .unwrap();
    client
        .delete(format!("{base}/api/discord/guild/RECLAIMGUILD"))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap();

    // Second owner can now claim the same guild_id.
    let body_b = register(&client, &base, "reclaim_b@test.local", "reclaim_b").await;
    let token_b = body_b["access_token"].as_str().unwrap().to_string();
    let user_b = uuid::Uuid::parse_str(body_b["user"]["id"].as_str().unwrap()).unwrap();
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id) \
         VALUES (gen_random_uuid(), $1, 'discord', 'discord-reclaim-B')",
    )
    .bind(user_b)
    .execute(&pool)
    .await
    .unwrap();

    let res = client
        .put(format!("{base}/api/discord/guild"))
        .bearer_auth(&token_b)
        .json(&json!({"guild_id": "RECLAIMGUILD", "channel_id": "ch-r2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "reclaim after soft-delete must succeed");
}

// ---------------------------------------------------------------------------
// Discord interactions endpoint — signature verification
// ---------------------------------------------------------------------------

/// With no public key configured the interactions endpoint must always return 401.
#[sqlx::test]
async fn interactions_without_public_key_returns_401(pool: PgPool) {
    // The default test server has an empty discord_bot_public_key (fail-closed).
    let base = spawn_test_server(pool).await;
    let client = Client::new();
    let res = client
        .post(format!("{base}/api/discord/interactions"))
        .json(&json!({"type": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 401, "missing public key must reject all interactions");
}

/// A request with a known key but an incorrect signature must return 401.
#[sqlx::test]
async fn interactions_with_wrong_signature_returns_401(pool: PgPool) {
    let signing_key = SigningKey::from_bytes(&[42u8; 32]);
    let pub_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let base = spawn_test_server_with_config(pool, AppConfig {
        discord_bot_public_key: pub_key_hex,
        ..default_test_config()
    })
    .await;
    let client = Client::new();

    let res = client
        .post(format!("{base}/api/discord/interactions"))
        .header("x-signature-ed25519", "ab".repeat(32)) // valid length, wrong content
        .header("x-signature-timestamp", "1234567890")
        .body(r#"{"type":1}"#)
        .header("content-type", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 401, "wrong signature must return 401");
}

/// A valid ping with a correct ed25519 signature must return a pong (type=1).
#[sqlx::test]
async fn interactions_ping_with_valid_signature_returns_pong(pool: PgPool) {
    let signing_key = SigningKey::from_bytes(&[99u8; 32]);
    let pub_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let base = spawn_test_server_with_config(pool, AppConfig {
        discord_bot_public_key: pub_key_hex,
        ..default_test_config()
    })
    .await;
    let client = Client::new();

    let body_bytes = br#"{"type":1}"#;
    let timestamp = "1234567890";
    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body_bytes);
    let sig_hex = hex::encode(signing_key.sign(&message).to_bytes());

    let res = client
        .post(format!("{base}/api/discord/interactions"))
        .header("x-signature-ed25519", sig_hex)
        .header("x-signature-timestamp", timestamp)
        .header("content-type", "application/json")
        .body(body_bytes.as_ref())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "valid ping must return 200");
    let resp: Value = res.json().await.unwrap();
    assert_eq!(resp["type"].as_u64().unwrap(), 1, "ping must respond with type=1 pong");
}
