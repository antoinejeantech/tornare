//! Tests for Battle.net OAuth features: disconnect guard and soft-delete reconnect behavior.
//!
//! Run with:
//!   cargo test --test battlenet

mod common;

use common::{register, spawn_test_server};
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Battle.net disconnect guard
// ---------------------------------------------------------------------------

/// A user whose only login method is Battle.net (no password) must receive a
/// 400 if they try to disconnect, so they cannot lock themselves out.
#[sqlx::test]
async fn disconnect_battlenet_is_blocked_when_user_has_no_password(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body = register(&client, &base, "bnetonly@test.local", "bnetonly").await;
    let token = body["access_token"].as_str().expect("must have token").to_string();
    let user_id = Uuid::parse_str(
        body["user"]["id"].as_str().expect("must have user.id"),
    )
    .expect("user.id must be a valid UUID");

    // Simulate a bnet-only user: insert a battlenet identity and remove the password.
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id)
         VALUES (gen_random_uuid(), $1, 'battlenet', 'test-sub-no-pwd')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to insert bnet identity");

    sqlx::query("UPDATE users SET password_hash = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("failed to clear password_hash");

    let res = client
        .delete(format!("{base}/api/auth/battlenet/disconnect"))
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

/// A user who has a password set can disconnect Battle.net without being blocked.
#[sqlx::test]
async fn disconnect_battlenet_succeeds_when_user_has_password(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body = register(&client, &base, "bnetpwd@test.local", "bnetpwd").await;
    let token = body["access_token"].as_str().expect("must have token").to_string();
    let user_id = Uuid::parse_str(
        body["user"]["id"].as_str().expect("must have user.id"),
    )
    .expect("user.id must be a valid UUID");

    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id)
         VALUES (gen_random_uuid(), $1, 'battlenet', 'test-sub-with-pwd')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to insert bnet identity");

    let res = client
        .delete(format!("{base}/api/auth/battlenet/disconnect"))
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

/// After a successful disconnect the identity row must be soft-deleted
/// (provider = 'battlenet_disconnected') rather than removed entirely,
/// so that the login flow can still recognise the returning user.
#[sqlx::test]
async fn disconnect_battlenet_soft_deletes_identity(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body = register(&client, &base, "bnetsoft@test.local", "bnetsoft").await;
    let token = body["access_token"].as_str().expect("must have token").to_string();
    let user_id = Uuid::parse_str(
        body["user"]["id"].as_str().expect("must have user.id"),
    )
    .expect("user.id must be a valid UUID");

    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id)
         VALUES (gen_random_uuid(), $1, 'battlenet', 'test-sub-soft-del')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to insert bnet identity");

    let res = client
        .delete(format!("{base}/api/auth/battlenet/disconnect"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("disconnect request failed");
    assert_eq!(res.status().as_u16(), 200, "disconnect must succeed");

    // The identity must now be soft-deleted, not removed.
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT provider FROM auth_identities
         WHERE user_id = $1 AND provider_user_id = 'test-sub-soft-del'",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .expect("DB query failed");

    let (provider,) = row.expect("identity row must still exist after disconnect");
    assert_eq!(
        provider, "battlenet_disconnected",
        "disconnected identity must have provider='battlenet_disconnected'"
    );
}

/// The `/api/auth/me` endpoint must expose `has_password: true` for a user
/// registered with email + password.
#[sqlx::test]
async fn me_endpoint_reports_has_password_for_password_account(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let body = register(&client, &base, "haspwd@test.local", "haspwd").await;
    let token = body["access_token"].as_str().expect("must have token").to_string();

    let res = client
        .get(format!("{base}/api/auth/me"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("GET /me request failed");
    assert_eq!(res.status().as_u16(), 200, "/me must return 200");

    let me: serde_json::Value = res.json().await.expect("response must be JSON");
    assert_eq!(
        me["has_password"],
        serde_json::json!(true),
        "/me must report has_password=true for a password-registered account"
    );
}

// ---------------------------------------------------------------------------
// Battle.net reconnect: disconnected sub can be claimed by a different user
// ---------------------------------------------------------------------------

/// When user A disconnects their Battle.net account, the identity row is
/// hard-deleted. Both `find_user_id_by_bnet_sub` (login flow) and any
/// subsequent connection check must return None for that sub.
#[sqlx::test]
async fn disconnected_bnet_sub_is_fully_removed(pool: PgPool) {
    let user_a_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name)
         VALUES ($1, 'usera-bnet@test.local', 'usera_bnet', 'User A')",
    )
    .bind(user_a_id)
    .execute(&pool)
    .await
    .expect("failed to insert user A");

    let sub = "reconnect-test-sub-99999";
    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id)
         VALUES (gen_random_uuid(), $1, 'battlenet', $2)",
    )
    .bind(user_a_id)
    .bind(sub)
    .execute(&pool)
    .await
    .expect("failed to insert bnet identity");

    // Hard-delete via remove_bnet_identity.
    tornare::features::auth::repo::remove_bnet_identity(&pool, user_a_id)
        .await
        .unwrap_or_else(|_| panic!("remove_bnet_identity must not error"));

    // Row must be fully gone — login flow returns None.
    let login_match = tornare::features::auth::repo::find_user_id_by_bnet_sub(&pool, sub)
        .await
        .unwrap_or_else(|_| panic!("find_user_id_by_bnet_sub must not error"));
    assert!(
        login_match.is_none(),
        "after disconnect the sub must not be found by the login flow"
    );

    // Row count in auth_identities for this user must be zero.
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM auth_identities WHERE provider_user_id = $1",
    )
    .bind(sub)
    .fetch_one(&pool)
    .await
    .expect("count query must succeed");
    assert_eq!(count.0, 0, "no auth_identity rows must remain after hard delete");
}

/// After a disconnect + reconnect via `ensure_bnet_identity`, the sub must be
/// active again (`battlenet_disconnected` row upgraded to `battlenet`).
#[sqlx::test]
async fn ensure_bnet_identity_inserts_new_active_row(pool: PgPool) {
    let user_a_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name)
         VALUES ($1, 'reactivate-bnet@test.local', 'reactivate_bnet', 'User Reactivate')",
    )
    .bind(user_a_id)
    .execute(&pool)
    .await
    .expect("failed to insert user");

    let sub = "reactivate-sub-88888";

    // No prior row — ensure_bnet_identity must insert a fresh active row.
    tornare::features::auth::repo::ensure_bnet_identity(&pool, user_a_id, sub)
        .await
        .unwrap_or_else(|_| panic!("ensure_bnet_identity must succeed"));

    let (provider,): (String,) = sqlx::query_as(
        "SELECT provider FROM auth_identities WHERE user_id = $1 AND provider_user_id = $2",
    )
    .bind(user_a_id)
    .bind(sub)
    .fetch_one(&pool)
    .await
    .expect("identity row must exist after ensure_bnet_identity");

    assert_eq!(
        provider, "battlenet",
        "ensure_bnet_identity must create an active 'battlenet' row"
    );
}

// ---------------------------------------------------------------------------
// Disconnect requires authentication
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn disconnect_battlenet_without_auth_is_rejected(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let res = client
        .delete(format!("{base}/api/auth/battlenet/disconnect"))
        .send()
        .await
        .expect("request must complete");

    assert_eq!(
        res.status().as_u16(),
        401,
        "disconnect without a token must return 401"
    );
}
