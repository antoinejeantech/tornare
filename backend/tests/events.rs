//! Tests for event-specific behavior: lifecycle, roster management, teams,
//! auto-balance, public signups, soft-delete, and ended-state visibility.
//!
//! Run with:
//!   cargo test --test events

mod common;

use common::{find_named_item, count_team_role, register, spawn_test_server};
use reqwest::Client;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Access control
// ---------------------------------------------------------------------------

/// Creating an event without a token must return 401.
#[sqlx::test]
async fn create_event_without_auth_is_rejected(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let res = client
        .post(format!("{base}/api/events"))
        .json(&json!({
            "name": "No Auth Event",
            "description": "Should fail.",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": false,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status().as_u16(),
        401,
        "unauthenticated create event must return 401"
    );
}

// ---------------------------------------------------------------------------
// Legacy data compatibility
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn legacy_flex_players_do_not_break_events_listing(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let owner = register(&client, &base, "owner-flex@test.local", "owner_flex").await;
    let token = owner["access_token"]
        .as_str()
        .expect("owner response must include access token")
        .to_string();

    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Legacy Flex Event",
            "description": "",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": false,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create event should return 200");
    let event: Value = res.json().await.unwrap();
    let event_id = event["id"].as_str().expect("event id missing").to_string();

    let res = client
        .post(format!("{base}/api/events/{event_id}/players"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Legacy Flex Player",
            "role": "DPS",
            "rank": "Gold"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "add player should return 200");
    let event: Value = res.json().await.unwrap();
    let player_id = find_named_item(&event["players"], "Legacy Flex Player")["id"]
        .as_str()
        .expect("player id missing")
        .to_string();
    let player_uuid = Uuid::parse_str(&player_id).expect("player id must be a valid uuid");

    sqlx::query("UPDATE event_players SET role = 'FLEX' WHERE id = $1")
        .bind(player_uuid)
        .execute(&pool)
        .await
        .expect("failed to update event_players role to FLEX");
    sqlx::query("UPDATE event_player_roles SET role = 'FLEX' WHERE event_player_id = $1")
        .bind(player_uuid)
        .execute(&pool)
        .await
        .expect("failed to update event_player_roles role to FLEX");

    let res = client
        .get(format!("{base}/api/events?page=1&per_page=12"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "events listing should tolerate legacy FLEX data");

    let payload: Value = res.json().await.unwrap();
    let listed_event = payload["items"]
        .as_array()
        .expect("items must be an array")
        .iter()
        .find(|item| item["id"].as_str() == Some(event_id.as_str()))
        .expect("expected legacy event in listing");
    let listed_player = find_named_item(&listed_event["players"], "Legacy Flex Player");
    assert_eq!(listed_player["role"].as_str().unwrap(), "DPS");

    let roles = listed_player["roles"].as_array().expect("player roles must be an array");
    assert_eq!(roles.len(), 3, "legacy FLEX role should expand to three preferences");
}

// ---------------------------------------------------------------------------
// Auto-balance
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn auto_balance_requires_exactly_two_teams(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let owner = register(&client, &base, "owner4@test.local", "owner4").await;
    assert!(owner["access_token"].is_string(), "owner registration failed: {owner}");
    let token = owner["access_token"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "One Team Balance Guard",
            "description": "",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": false,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create event should return 200");
    let event: Value = res.json().await.unwrap();
    let event_id = event["id"].as_str().expect("event id missing").to_string();

    let res = client
        .post(format!("{base}/api/events/{event_id}/teams"))
        .bearer_auth(&token)
        .json(&json!({ "name": "Solo" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "team creation should succeed");

    let res = client
        .post(format!("{base}/api/events/{event_id}/teams/auto-balance"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 400, "auto-balance should require exactly two teams");
}

/// Auto-balance in 5v5 must always produce 1 Tank, 2 DPS, 2 Supports per team.
#[sqlx::test]
async fn auto_balance_5v5_enforces_exact_role_shape(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let owner = register(&client, &base, "owner3@test.local", "owner3").await;
    assert!(owner["access_token"].is_string(), "owner registration failed: {owner}");
    let token = owner["access_token"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Balance Regression",
            "description": "",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": false,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let event: Value = res.json().await.unwrap();
    let event_id = event["id"].as_str().unwrap().to_string();

    for team_name in ["Alpha", "Bravo"] {
        let res = client
            .post(format!("{base}/api/events/{event_id}/teams"))
            .bearer_auth(&token)
            .json(&json!({ "name": team_name }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status().as_u16(), 200, "team creation should succeed");
    }

    let players = [
        ("Aegis", vec![("Tank", "Gold")]),
        ("Blitz", vec![("DPS", "Diamond"), ("Tank", "Platinum")]),
        ("Cipher", vec![("DPS", "Diamond"), ("Support", "Diamond")]),
        ("Drift", vec![("DPS", "Platinum")]),
        ("Echo", vec![("DPS", "Gold")]),
        ("Flux", vec![("DPS", "Gold")]),
        ("Glow", vec![("Support", "Platinum")]),
        ("Halo", vec![("Support", "Gold")]),
        ("Iris", vec![("Support", "Diamond")]),
        ("Jolt", vec![("Support", "Gold"), ("Tank", "Gold")]),
    ];

    for (name, roles) in players {
        let res = client
            .post(format!("{base}/api/events/{event_id}/players"))
            .bearer_auth(&token)
            .json(&json!({
                "name": name,
                "role": roles[0].0,
                "rank": roles[0].1,
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status().as_u16(), 200, "adding player {name} should succeed");
        let event: Value = res.json().await.unwrap();
        let player_id = find_named_item(&event["players"], name)["id"].as_str().unwrap().to_string();

        if roles.len() > 1 {
            let role_payload: Vec<Value> = roles
                .iter()
                .map(|(role, rank)| json!({ "role": role, "rank": rank }))
                .collect();
            let res = client
                .put(format!("{base}/api/events/{event_id}/players/{player_id}"))
                .bearer_auth(&token)
                .json(&json!({
                    "name": name,
                    "role": roles[0].0,
                    "rank": roles[0].1,
                    "roles": role_payload,
                }))
                .send()
                .await
                .unwrap();
            assert_eq!(res.status().as_u16(), 200, "updating roles for {name} should succeed");
        }
    }

    let res = client
        .post(format!("{base}/api/events/{event_id}/teams/auto-balance"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "auto-balance should succeed");
    let payload: Value = res.json().await.unwrap();
    let event = &payload["event"];

    for team in event["teams"].as_array().unwrap() {
        let team_id = team["id"].as_str().unwrap();
        assert_eq!(count_team_role(event, team_id, "Tank"), 1, "team must have exactly 1 tank");
        assert_eq!(count_team_role(event, team_id, "DPS"), 2, "team must have exactly 2 DPS");
        assert_eq!(count_team_role(event, team_id, "Support"), 2, "team must have exactly 2 supports");
    }
}

// ---------------------------------------------------------------------------
// Team assignment
// ---------------------------------------------------------------------------

/// Team assignment with explicit assigned_role must not change the player's preferred role.
#[sqlx::test]
async fn team_assignment_with_role_does_not_change_preferred_role(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let owner = register(&client, &base, "owner2@test.local", "owner2").await;
    assert!(owner["access_token"].is_string(), "owner registration failed: {owner}");
    let token = owner["access_token"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Role Test Event",
            "description": "",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": false,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    let event: Value = res.json().await.unwrap();
    let event_id = event["id"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{base}/api/events/{event_id}/players"))
        .bearer_auth(&token)
        .json(&json!({"name": "Dave", "role": "Tank", "rank": "Gold"}))
        .send()
        .await
        .unwrap();
    let event: Value = res.json().await.unwrap();
    let player_id = find_named_item(&event["players"], "Dave")["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = client
        .post(format!("{base}/api/events/{event_id}/teams"))
        .bearer_auth(&token)
        .json(&json!({"name": "Alpha"}))
        .send()
        .await
        .unwrap();
    let event: Value = res.json().await.unwrap();
    let team_id = find_named_item(&event["teams"], "Alpha")["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = client
        .post(format!("{base}/api/events/{event_id}/team-members"))
        .bearer_auth(&token)
        .json(&json!({
            "player_id": player_id,
            "team_id": team_id,
            "assigned_role": "DPS",
            "assigned_rank": "Gold"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "assign player with role should succeed");
    let event: Value = res.json().await.unwrap();
    let dave = find_named_item(&event["players"], "Dave");

    assert_eq!(dave["role"].as_str().unwrap(), "Tank", "preferred role must not change");
    assert_eq!(dave["assigned_role"].as_str().unwrap(), "DPS", "assigned_role must be DPS");
}

// ---------------------------------------------------------------------------
// Public signup flow
// ---------------------------------------------------------------------------

/// Public signup flow: submit a request, then the owner accepts it.
#[sqlx::test]
async fn public_signup_request_can_be_submitted_and_accepted(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    let owner = register(&client, &base, "owner@test.local", "owner").await;
    assert!(owner["access_token"].is_string(), "owner registration failed: {owner}");
    let token = owner["access_token"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Open PUG",
            "description": "",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": true,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create event should return 200");
    let event: Value = res.json().await.unwrap();
    let event_id = event["id"].as_str().expect("event id missing").to_string();

    let res = client
        .get(format!("{base}/api/events/{event_id}/signup-link"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let link: Value = res.json().await.unwrap();
    let signup_token = link["signup_token"].as_str().expect("signup_token missing").to_string();

    let res = client
        .post(format!("{base}/api/public/event-signups/{signup_token}/requests"))
        .json(&json!({
            "name": "Carol",
            "roles": [{"role": "Support", "rank": "Platinum"}]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "signup request should be accepted");

    // Duplicate name while pending must be rejected.
    let res = client
        .post(format!("{base}/api/public/event-signups/{signup_token}/requests"))
        .json(&json!({
            "name": "Carol",
            "roles": [{"role": "Support", "rank": "Platinum"}]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 400, "duplicate pending signup must return 400");

    let res = client
        .get(format!("{base}/api/events/{event_id}/signup-requests"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let requests: Value = res.json().await.unwrap();
    let request_id = find_named_item(&requests, "Carol")["id"]
        .as_str()
        .expect("request id missing")
        .to_string();

    let res = client
        .post(format!("{base}/api/events/{event_id}/signup-requests/{request_id}/accept"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "accept signup should return 200");
    let event: Value = res.json().await.unwrap();
    assert_eq!(
        event["players"].as_array().unwrap().len(),
        1,
        "event should have one player after acceptance"
    );
    assert_eq!(
        find_named_item(&event["players"], "Carol")["role"].as_str().unwrap(),
        "Support"
    );
}

// ---------------------------------------------------------------------------
// Soft-delete and ended-state visibility
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn deleting_event_soft_deletes_it_and_hides_it(pool: PgPool) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let owner = register(&client, &base, "softdelete@test.local", "softdelete").await;
    assert!(owner["access_token"].is_string(), "owner registration failed: {owner}");
    let token = owner["access_token"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Soft Delete Me",
            "description": "",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": false,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create event should return 200");
    let event: Value = res.json().await.unwrap();
    let event_id = event["id"].as_str().unwrap().to_string();
    let event_uuid = Uuid::parse_str(&event_id).expect("event id must be a uuid");

    let res = client
        .delete(format!("{base}/api/events/{event_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "delete event should return 200");

    let deleted_at: Option<sqlx::types::time::OffsetDateTime> =
        sqlx::query_scalar("SELECT deleted_at FROM events WHERE id = $1")
            .bind(event_uuid)
            .fetch_one(&pool)
            .await
            .expect("deleted event row should still exist");
    assert!(deleted_at.is_some(), "soft-deleted event must retain row with deleted_at set");

    let res = client
        .get(format!("{base}/api/events/{event_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 404, "deleted event should not be directly readable");

    let res = client
        .get(format!("{base}/api/events?page=1&per_page=12"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "events listing should still succeed");
    let payload: Value = res.json().await.unwrap();
    let still_listed = payload["items"]
        .as_array()
        .expect("items must be an array")
        .iter()
        .any(|item| item["id"].as_str() == Some(event_id.as_str()));
    assert!(!still_listed, "soft-deleted event must be hidden from listings");
}

#[sqlx::test]
async fn ending_event_hides_it_from_default_listing_allows_direct_access_and_can_be_reopened(
    pool: PgPool,
) {
    let base = spawn_test_server(pool.clone()).await;
    let client = Client::new();

    let owner = register(&client, &base, "endedvis@test.local", "endedvis").await;
    assert!(owner["access_token"].is_string(), "owner registration failed: {owner}");
    let token = owner["access_token"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Ended Visibility Test",
            "description": "",
            "event_type": "PUG",
            "format": "5v5",
            "public_signup_enabled": false,
            "max_players": 10
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create event failed");
    let event: Value = res.json().await.unwrap();
    let event_id = event["id"].as_str().unwrap().to_string();

    let res = client
        .put(format!("{base}/api/events/{event_id}/ended"))
        .bearer_auth(&token)
        .json(&json!({ "ended": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "set-ended should return 200");
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["is_ended"], json!(true), "event should be marked as ended");

    let res = client
        .get(format!("{base}/api/events?page=1&per_page=12"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "events listing should succeed");
    let payload: Value = res.json().await.unwrap();
    let in_default_listing = payload["items"]
        .as_array()
        .expect("items must be an array")
        .iter()
        .any(|item| item["id"].as_str() == Some(event_id.as_str()));
    assert!(!in_default_listing, "ended event must not appear in the default (non-ended) listing");

    let res = client
        .get(format!("{base}/api/events?page=1&per_page=12&ended_only=true"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "ended-only listing should succeed");
    let payload: Value = res.json().await.unwrap();
    let in_ended_listing = payload["items"]
        .as_array()
        .expect("items must be an array")
        .iter()
        .any(|item| item["id"].as_str() == Some(event_id.as_str()));
    assert!(in_ended_listing, "ended event must appear in the ended-only listing");

    let res = client
        .get(format!("{base}/api/events/{event_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "ended event should still be directly accessible");

    let res = client
        .put(format!("{base}/api/events/{event_id}/ended"))
        .bearer_auth(&token)
        .json(&json!({ "ended": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "reopen should return 200");
    let reopened: Value = res.json().await.unwrap();
    assert_eq!(reopened["is_ended"], json!(false), "event should be marked as not ended after reopen");

    let res = client
        .get(format!("{base}/api/events?page=1&per_page=12"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "events listing should succeed after reopen");
    let payload: Value = res.json().await.unwrap();
    let back_in_listing = payload["items"]
        .as_array()
        .expect("items must be an array")
        .iter()
        .any(|item| item["id"].as_str() == Some(event_id.as_str()));
    assert!(back_in_listing, "reopened event must reappear in the default listing");
}
