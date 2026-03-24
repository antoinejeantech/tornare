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
