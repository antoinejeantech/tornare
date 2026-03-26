//! End-to-end happy-path flow test: register → login → create event →
//! manage roster/teams/match → fetch event.
//!
//! This single test deliberately spans multiple domains (auth + events) to
//! verify that the full user journey works end-to-end.  Domain-specific tests
//! live in the dedicated `users`, `events`, and `battlenet` test binaries.
//!
//! Each test function uses `#[sqlx::test]` which automatically:
//!   - reads DATABASE_URL from the environment
//!   - creates a fresh, randomly-named temporary Postgres database
//!   - runs all migrations in backend/migrations/
//!   - provides a ready-to-use PgPool
//!   - drops the temporary database when the test finishes
//!
//! Run with:
//!   cargo test --test e2e
//!
//! Or with an explicit database URL:
//!   DATABASE_URL=postgres://postgres:postgres@localhost/tornare cargo test --test e2e

mod common;

use common::{find_named_item, login, register, spawn_test_server};
use reqwest::Client;
use serde_json::{json, Value};
use sqlx::PgPool;

/// Happy-path flow: register → login → create event → manage roster/teams/match → fetch event.
#[sqlx::test]
async fn user_can_register_login_create_and_read_event(pool: PgPool) {
    let base = spawn_test_server(pool).await;
    let client = Client::new();

    // 1. Register — should return an AuthResponse with tokens + user.
    let body = register(&client, &base, "alice@test.local", "alice").await;
    assert!(
        body["access_token"].is_string(),
        "register must return an access_token; got: {body}"
    );
    let user_id = body["user"]["id"]
        .as_str()
        .expect("register response must include user.id")
        .to_string();

    // 2. Login — independent call that shows credentials work.
    let body = login(&client, &base, "alice@test.local").await;
    assert!(
        body["access_token"].is_string(),
        "login must return an access_token; got: {body}"
    );
    let token = body["access_token"].as_str().unwrap().to_string();

    // 3. GET /api/auth/me — verify the token is accepted.
    let res = client
        .get(format!("{base}/api/auth/me"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "/api/auth/me should return 200");
    let me: Value = res.json().await.unwrap();
    assert_eq!(me["id"].as_str().unwrap(), user_id, "me.id must match registered user");

    // 4. Create event.
    let res = client
        .post(format!("{base}/api/events"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Grand Tournament",
            "description": "An automated end-to-end test event.",
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
    let event_id = event["id"]
        .as_str()
        .expect("event response must contain id")
        .to_string();
    assert_eq!(event["name"].as_str().unwrap(), "Grand Tournament");
    assert_eq!(event["players"].as_array().unwrap().len(), 0);
    assert_eq!(event["teams"].as_array().unwrap().len(), 0);
    assert_eq!(event["matches"].as_array().unwrap().len(), 0);

    // 5. Add two players to the event roster.
    let res = client
        .post(format!("{base}/api/events/{event_id}/players"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Alice Tank",
            "role": "Tank",
            "rank": "Gold"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "add player should return 200");
    let event: Value = res.json().await.unwrap();
    assert_eq!(event["players"].as_array().unwrap().len(), 1);

    let res = client
        .post(format!("{base}/api/events/{event_id}/players"))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Bob Support",
            "role": "Support",
            "rank": "Diamond"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "add second player should return 200");
    let event: Value = res.json().await.unwrap();
    assert_eq!(event["players"].as_array().unwrap().len(), 2);

    let player_a_id = find_named_item(&event["players"], "Alice Tank")["id"]
        .as_str()
        .expect("player A id missing")
        .to_string();
    let player_b_id = find_named_item(&event["players"], "Bob Support")["id"]
        .as_str()
        .expect("player B id missing")
        .to_string();

    // 6. Create two teams.
    let res = client
        .post(format!("{base}/api/events/{event_id}/teams"))
        .bearer_auth(&token)
        .json(&json!({ "name": "Blue Team" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create team should return 200");

    let res = client
        .post(format!("{base}/api/events/{event_id}/teams"))
        .bearer_auth(&token)
        .json(&json!({ "name": "Red Team" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create second team should return 200");
    let event: Value = res.json().await.unwrap();
    assert_eq!(event["teams"].as_array().unwrap().len(), 2);

    let team_a_id = find_named_item(&event["teams"], "Blue Team")["id"]
        .as_str()
        .expect("team A id missing")
        .to_string();
    let team_b_id = find_named_item(&event["teams"], "Red Team")["id"]
        .as_str()
        .expect("team B id missing")
        .to_string();

    // 7. Assign players to teams.
    let res = client
        .post(format!("{base}/api/events/{event_id}/team-members"))
        .bearer_auth(&token)
        .json(&json!({
            "player_id": player_a_id,
            "team_id": team_a_id
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "assign first player should return 200");

    let res = client
        .post(format!("{base}/api/events/{event_id}/team-members"))
        .bearer_auth(&token)
        .json(&json!({
            "player_id": player_b_id,
            "team_id": team_b_id
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "assign second player should return 200");
    let event: Value = res.json().await.unwrap();

    let blue_team_members = find_named_item(&event["teams"], "Blue Team")["player_ids"]
        .as_array()
        .expect("blue team player_ids must be an array");
    assert!(
        blue_team_members.iter().any(|item| item.as_str() == Some(player_a_id.as_str())),
        "blue team should contain player A"
    );

    // 8. Create a match for the event.
    let res = client
        .post(format!("{base}/api/events/{event_id}/matches"))
        .bearer_auth(&token)
        .json(&json!({
            "title": "Week 1 Showcase",
            "map": "Nepal",
            "start_date": null
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "create match should return 200");
    let created_match: Value = res.json().await.unwrap();
    let match_id = created_match["id"]
        .as_str()
        .expect("match response must contain id")
        .to_string();
    assert_eq!(created_match["status"].as_str().unwrap(), "OPEN");

    let res = client
        .post(format!("{base}/api/events/{event_id}/matches/{match_id}/matchup"))
        .bearer_auth(&token)
        .json(&json!({
            "team_a_id": team_a_id
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status().as_u16(),
        400,
        "set matchup should reject omitted team fields"
    );

    // 9. Set the matchup with the two teams.
    let res = client
        .post(format!("{base}/api/events/{event_id}/matches/{match_id}/matchup"))
        .bearer_auth(&token)
        .json(&json!({
            "team_a_id": team_a_id,
            "team_b_id": team_b_id
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "set matchup should return 200");
    let updated_match: Value = res.json().await.unwrap();
    assert_eq!(updated_match["status"].as_str().unwrap(), "OPEN");
    assert_eq!(updated_match["team_a_id"].as_str().unwrap(), team_a_id);
    assert_eq!(updated_match["team_b_id"].as_str().unwrap(), team_b_id);

    // 10. Report a winner.
    let res = client
        .post(format!("{base}/api/events/{event_id}/matches/{match_id}/winner"))
        .bearer_auth(&token)
        .json(&json!({
            "winner_team_id": team_a_id
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "report winner should return 200");
    let completed_match: Value = res.json().await.unwrap();
    assert_eq!(completed_match["status"].as_str().unwrap(), "COMPLETED");
    assert_eq!(completed_match["winner_team_id"].as_str().unwrap(), team_a_id);

    // 11. Fetch the event and verify the full managed state.
    let res = client
        .get(format!("{base}/api/events/{event_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "get event should return 200");
    let fetched: Value = res.json().await.unwrap();
    assert_eq!(fetched["id"].as_str().unwrap(), event_id);
    assert_eq!(fetched["name"].as_str().unwrap(), "Grand Tournament");
    assert_eq!(fetched["players"].as_array().unwrap().len(), 2);
    assert_eq!(fetched["teams"].as_array().unwrap().len(), 2);
    assert_eq!(fetched["matches"].as_array().unwrap().len(), 1);
    assert_eq!(fetched["matches"][0]["winner_team_id"].as_str().unwrap(), team_a_id);
}
