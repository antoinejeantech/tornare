//! Tests for user management: registration, profile editing, and admin account deletion.
//!
//! Run with:
//!   cargo test --test users

mod common;

use common::{promote_to_admin, register, spawn_test_server};
use reqwest::Client;
use serde_json::{json, Value};
use sqlx::PgPool;

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Duplicate email registration must be rejected.
#[sqlx::test]
async fn register_with_duplicate_email_is_rejected(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let first = register(&client, &base, "bob@test.local", "bob").await;
    assert!(
        first["access_token"].is_string(),
        "first registration must return an access_token; got: {first}"
    );

    let res = client
        .post(format!("{base}/api/auth/register"))
        .json(&json!({
            "email": "bob@test.local",
            "password": "Password123!",
            "password_confirm": "Password123!",
            "username": "bob2",
            "display_name": "Bob Again"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status().as_u16(),
        400,
        "duplicate email must return 400 Bad Request"
    );
}

// ---------------------------------------------------------------------------
// Profile editing
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn admin_can_edit_another_users_profile(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let admin = register(&client, &base, "admin@test.local", "admin_user").await;
    let target = register(&client, &base, "target@test.local", "target_user").await;

    let admin_id = admin["user"]["id"]
        .as_str()
        .expect("admin response must include user id")
        .to_string();
    let admin_token = admin["access_token"]
        .as_str()
        .expect("admin response must include access token")
        .to_string();
    let target_id = target["user"]["id"]
        .as_str()
        .expect("target response must include user id")
        .to_string();

    promote_to_admin(&pool, &admin_id).await;

    let res = client
        .put(format!("{base}/api/users/{target_id}"))
        .bearer_auth(&admin_token)
        .json(&json!({
            "username": "target_admin_edited",
            "display_name": "Edited By Admin",
            "email": "target-edited@test.local",
            "battletag": null,
            "rank_tank": "Gold",
            "rank_dps": "Diamond",
            "rank_support": "Platinum",
            "new_password": null,
            "new_password_confirm": null
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "admin profile edit should succeed");

    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["id"].as_str().unwrap(), target_id);
    assert_eq!(updated["username"].as_str().unwrap(), "target_admin_edited");
    assert_eq!(updated["display_name"].as_str().unwrap(), "Edited By Admin");
    assert_eq!(updated["email"].as_str().unwrap(), "target-edited@test.local");
    assert_eq!(updated["rank_tank"].as_str().unwrap(), "Gold");
    assert_eq!(updated["rank_dps"].as_str().unwrap(), "Diamond");
    assert_eq!(updated["rank_support"].as_str().unwrap(), "Platinum");

    let res = client
        .get(format!("{base}/api/users/{admin_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "admin profile should still be readable");
    let admin_profile: Value = res.json().await.unwrap();
    assert_eq!(admin_profile["username"].as_str().unwrap(), "admin_user");
}

// ---------------------------------------------------------------------------
// Admin account deletion
// ---------------------------------------------------------------------------

/// An admin can delete another user's account.
/// After deletion, fetching the profile by ID must return 404.
#[sqlx::test]
async fn admin_can_delete_another_users_account(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let admin = register(&client, &base, "deladmin@test.local", "deladmin").await;
    let target = register(&client, &base, "deltarget@test.local", "deltarget").await;

    let admin_id = admin["user"]["id"].as_str().expect("admin must have id").to_string();
    let admin_token = admin["access_token"].as_str().expect("admin must have token").to_string();
    let target_id = target["user"]["id"].as_str().expect("target must have id").to_string();

    promote_to_admin(&pool, &admin_id).await;

    // Admin deletes the target account.
    let res = client
        .delete(format!("{base}/api/users/{target_id}"))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("delete request must complete");
    assert_eq!(
        res.status().as_u16(),
        200,
        "admin delete must return 200; got: {}",
        res.status()
    );

    // Deleted account must no longer be reachable.
    let res = client
        .get(format!("{base}/api/users/{target_id}"))
        .send()
        .await
        .expect("profile lookup must complete");
    assert_eq!(
        res.status().as_u16(),
        404,
        "deleted user profile must return 404"
    );
}

#[sqlx::test]
async fn admin_cannot_delete_their_own_account(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let admin = register(&client, &base, "selfdelete@test.local", "selfdelete").await;
    let admin_id = admin["user"]["id"].as_str().expect("admin must have id").to_string();
    let admin_token = admin["access_token"].as_str().expect("admin must have token").to_string();

    promote_to_admin(&pool, &admin_id).await;

    let res = client
        .delete(format!("{base}/api/users/{admin_id}"))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("delete request must complete");
    assert_eq!(res.status().as_u16(), 403, "admin self-delete must be rejected");
}

/// A regular (non-admin) user must not be able to delete another account.
#[sqlx::test]
async fn non_admin_cannot_delete_user_account(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let user_a = register(&client, &base, "nodela@test.local", "nodela").await;
    let user_b = register(&client, &base, "nodelb@test.local", "nodelb").await;

    let token_a = user_a["access_token"].as_str().expect("user A must have token").to_string();
    let id_b = user_b["user"]["id"].as_str().expect("user B must have id").to_string();

    let res = client
        .delete(format!("{base}/api/users/{id_b}"))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("delete request must complete");
    assert_eq!(
        res.status().as_u16(),
        403,
        "non-admin delete must return 403"
    );
}

/// Calling the delete endpoint without a token must return 401.
#[sqlx::test]
async fn unauthenticated_cannot_delete_user_account(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let target = register(&client, &base, "unauthdeleted@test.local", "unauthdeleted").await;
    let target_id = target["user"]["id"].as_str().expect("must have id").to_string();

    let res = client
        .delete(format!("{base}/api/users/{target_id}"))
        .send()
        .await
        .expect("request must complete");
    assert_eq!(
        res.status().as_u16(),
        401,
        "unauthenticated delete must return 401"
    );
}

/// Attempting to delete a user that does not exist must return 404 (not a
/// server error).
#[sqlx::test]
async fn admin_delete_of_nonexistent_user_returns_404(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let admin = register(&client, &base, "del404admin@test.local", "del404admin").await;
    let admin_id = admin["user"]["id"].as_str().expect("must have id").to_string();
    let admin_token = admin["access_token"].as_str().expect("must have token").to_string();

    promote_to_admin(&pool, &admin_id).await;

    // Use a valid but non-existent UUID.
    let ghost_id = uuid::Uuid::new_v4();

    let res = client
        .delete(format!("{base}/api/users/{ghost_id}"))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("request must complete");
    assert_eq!(
        res.status().as_u16(),
        404,
        "deleting a non-existent user must return 404"
    );
}

// ---------------------------------------------------------------------------
// Avatar picker
// ---------------------------------------------------------------------------

/// Updating to a valid preset path must succeed and return the new avatar_url.
#[sqlx::test]
async fn set_avatar_to_valid_preset_succeeds(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let user = register(&client, &base, "avatarok@test.local", "avatarok").await;
    let user_id = user["user"]["id"].as_str().expect("must have id").to_string();
    let token = user["access_token"].as_str().expect("must have token").to_string();

    let res = client
        .patch(format!("{base}/api/users/{user_id}/avatar"))
        .bearer_auth(&token)
        .json(&json!({ "avatar_url": "/avatars/tracer.webp" }))
        .send()
        .await
        .expect("request must complete");
    assert_eq!(res.status().as_u16(), 200, "valid preset must return 200");

    let body: Value = res.json().await.expect("must return JSON");
    assert_eq!(
        body["avatar_url"].as_str(),
        Some("/avatars/tracer.webp"),
        "returned avatar_url must match the preset"
    );
}

/// Submitting an arbitrary external URL must be rejected with 400.
#[sqlx::test]
async fn set_avatar_to_arbitrary_url_is_rejected(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let user = register(&client, &base, "avatarbad@test.local", "avatarbad").await;
    let user_id = user["user"]["id"].as_str().expect("must have id").to_string();
    let token = user["access_token"].as_str().expect("must have token").to_string();

    let res = client
        .patch(format!("{base}/api/users/{user_id}/avatar"))
        .bearer_auth(&token)
        .json(&json!({ "avatar_url": "https://evil.example/steal.png" }))
        .send()
        .await
        .expect("request must complete");
    assert_eq!(
        res.status().as_u16(),
        400,
        "arbitrary URL must be rejected with 400"
    );
}

/// Sending null must reset the avatar_url to null (back to initials fallback).
#[sqlx::test]
async fn set_avatar_to_null_resets_avatar(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let user = register(&client, &base, "avatarnull@test.local", "avatarnull").await;
    let user_id = user["user"]["id"].as_str().expect("must have id").to_string();
    let token = user["access_token"].as_str().expect("must have token").to_string();

    // First set a preset.
    client
        .patch(format!("{base}/api/users/{user_id}/avatar"))
        .bearer_auth(&token)
        .json(&json!({ "avatar_url": "/avatars/mercy.webp" }))
        .send()
        .await
        .expect("request must complete");

    // Then reset to null.
    let res = client
        .patch(format!("{base}/api/users/{user_id}/avatar"))
        .bearer_auth(&token)
        .json(&json!({ "avatar_url": null }))
        .send()
        .await
        .expect("request must complete");
    assert_eq!(res.status().as_u16(), 200, "null reset must return 200");

    let body: Value = res.json().await.expect("must return JSON");
    assert!(
        body["avatar_url"].is_null(),
        "avatar_url must be null after reset; got: {}",
        body["avatar_url"]
    );
}

/// A user must not be able to change another user's avatar.
#[sqlx::test]
async fn user_cannot_set_another_users_avatar(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let alice = register(&client, &base, "alice_av@test.local", "alice_av").await;
    let bob = register(&client, &base, "bob_av@test.local", "bob_av").await;

    let bob_id = bob["user"]["id"].as_str().expect("must have id").to_string();
    let alice_token = alice["access_token"].as_str().expect("must have token").to_string();

    let res = client
        .patch(format!("{base}/api/users/{bob_id}/avatar"))
        .bearer_auth(&alice_token)
        .json(&json!({ "avatar_url": "/avatars/genji.webp" }))
        .send()
        .await
        .expect("request must complete");
    assert_eq!(
        res.status().as_u16(),
        403,
        "changing another user's avatar must return 403"
    );
}
