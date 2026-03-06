use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    db::{i32_to_usize, i64_to_usize, load_event, load_match},
    errors::{bad_request, forbidden, internal_error, not_found, unauthorized, ApiError, ApiResult},
    models::{
        AddPlayerInput, AssignEventPlayerTeamInput, AuthResponse, AuthUser, CreateEventInput,
        CreateEventMatchInput, CreateEventTeamInput, CreateMatchInput, Event, LoginInput, LogoutInput,
        Match, MessageResponse, RefreshInput, RegisterInput, SetMatchupInput, UpdateEventInput,
        UpdateEventPlayerInput, UpdateEventTeamInput, OVERWATCH_RANKS,
    },
    state::AppState,
};

const ACCESS_TOKEN_TTL_SECONDS: usize = 15 * 60;

#[derive(Serialize, Deserialize)]
struct AccessClaims {
    sub: String,
    exp: usize,
    token_type: String,
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn hello() -> Json<MessageResponse> {
    Json(MessageResponse {
        message: "Hello from Rust backend!".to_string(),
    })
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterInput>,
) -> ApiResult<AuthResponse> {
    validate_register_input(&payload)?;

    let normalized_email = normalize_email(&payload.email);
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&normalized_email)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if existing.is_some() {
        return Err(bad_request("Email is already registered"));
    }

    let password_hash = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(&normalized_email)
    .bind(password_hash)
    .bind(payload.display_name.trim())
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO auth_identities (id, user_id, provider, provider_user_id, email_from_provider)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind("local")
    .bind(&normalized_email)
    .bind(&normalized_email)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    sqlx::query("INSERT INTO user_roles (id, user_id, role) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind("user")
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    let user = get_auth_user_by_id(&state, user_id).await?;
    let response = issue_auth_response(&state, user, None).await?;

    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginInput>,
) -> ApiResult<AuthResponse> {
    let normalized_email = normalize_email(&payload.email);

    if normalized_email.is_empty() || payload.password.is_empty() {
        return Err(bad_request("Email and password are required"));
    }

    let user_row = sqlx::query(
        "SELECT id, email, password_hash, display_name, is_active FROM users WHERE email = $1",
    )
    .bind(&normalized_email)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    let Some(user_row) = user_row else {
        return Err(unauthorized("Invalid email or password"));
    };

    let is_active: bool = user_row.get("is_active");
    if !is_active {
        return Err(forbidden("User account is inactive"));
    }

    let Some(password_hash) = user_row.get::<Option<String>, _>("password_hash") else {
        return Err(unauthorized("Invalid email or password"));
    };

    verify_password(&password_hash, &payload.password)?;

    let user = AuthUser {
        id: user_row.get("id"),
        email: user_row.get("email"),
        display_name: user_row.get("display_name"),
    };

    let response = issue_auth_response(&state, user, None).await?;
    Ok(Json(response))
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<AuthUser> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    let user = get_auth_user_by_id(&state, user_id).await?;
    Ok(Json(user))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshInput>,
) -> ApiResult<AuthResponse> {
    if payload.refresh_token.trim().is_empty() {
        return Err(bad_request("Refresh token is required"));
    }

    let refresh_hash = hash_refresh_token(&payload.refresh_token);
    let session_row = sqlx::query(
        "SELECT id, user_id
         FROM auth_sessions
         WHERE refresh_token_hash = $1
           AND revoked_at IS NULL
           AND expires_at > NOW()",
    )
    .bind(refresh_hash)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    let Some(session_row) = session_row else {
        return Err(unauthorized("Invalid or expired refresh token"));
    };

    let session_id: Uuid = session_row.get("id");
    let user_id: Uuid = session_row.get("user_id");

    let user = get_auth_user_by_id(&state, user_id).await?;
    let response = issue_auth_response(&state, user, Some(session_id)).await?;

    Ok(Json(response))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutInput>,
) -> ApiResult<MessageResponse> {
    if payload.refresh_token.trim().is_empty() {
        return Err(bad_request("Refresh token is required"));
    }

    let refresh_hash = hash_refresh_token(&payload.refresh_token);
    sqlx::query("UPDATE auth_sessions SET revoked_at = NOW() WHERE refresh_token_hash = $1")
        .bind(refresh_hash)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok(Json(MessageResponse {
        message: "Logged out".to_string(),
    }))
}

pub async fn list_matches(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Vec<Match>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;

    let match_rows = sqlx::query(
        "SELECT em.id
         FROM event_matches em
         INNER JOIN event_memberships m ON m.event_id = em.event_id
         WHERE m.user_id = $1
         ORDER BY em.id DESC",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut matches = Vec::with_capacity(match_rows.len());
    for row in match_rows {
        let match_id: Uuid = row.get("id");
        matches.push(load_match(&state.pool, match_id).await?);
    }

    Ok(Json(matches))
}

pub async fn list_events(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Vec<Event>> {
    let user_id = require_authenticated_user_id(&state, &headers)?;

    let event_rows = sqlx::query(
        "SELECT e.id
         FROM events e
         INNER JOIN event_memberships m ON m.event_id = e.id
         WHERE m.user_id = $1
         ORDER BY e.id DESC",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut events = Vec::with_capacity(event_rows.len());
    for row in event_rows {
        let event_id: Uuid = row.get("id");
        events.push(load_event(&state.pool, event_id).await?);
    }

    Ok(Json(events))
}

pub async fn get_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_view_access(&state, event_id, user_id).await?;

    let event = load_event(&state.pool, event_id).await?;
    Ok(Json(event))
}

pub async fn create_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    validate_create_event_input(&payload)?;

    let event_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO events (id, name, event_type, max_players) VALUES ($1, $2, $3, $4)",
    )
    .bind(event_id)
    .bind(payload.name.trim())
    .bind(payload.event_type.as_db_value())
    .bind(i32::from(payload.max_players))
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO event_memberships (id, event_id, user_id, role) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(user_id)
    .bind("owner")
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let event = load_event(&state.pool, event_id).await?;

    Ok(Json(event))
}

pub async fn update_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;
    validate_update_event_input(&payload)?;

    let updated = sqlx::query(
        "UPDATE events SET name = $1, event_type = $2, max_players = $3 WHERE id = $4 RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.event_type.as_db_value())
    .bind(i32::from(payload.max_players))
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    if updated.is_none() {
        return Err(not_found("Event not found"));
    }

    let event = load_event(&state.pool, event_id).await?;
    Ok(Json(event))
}

pub async fn delete_event(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_owner_access(&state, event_id, user_id).await?;

    let result = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(event_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Event not found"));
    }

    Ok(Json(MessageResponse {
        message: "Event deleted".to_string(),
    }))
}

pub async fn create_event_match(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventMatchInput>,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;

    let event_row = sqlx::query("SELECT max_players FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    let Some(event_row) = event_row else {
        return Err(not_found("Event not found"));
    };

    let create_match = CreateMatchInput {
        title: payload.title,
        map: payload.map,
        max_players: crate::db::i32_to_u8(event_row.get::<i32, _>("max_players"), "max_players")?,
    };

    let created_match = create_match_record(&state, create_match, event_id).await?;
    Ok(Json(created_match))
}

pub async fn add_event_player(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddPlayerInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;
    validate_add_player_input(&payload)?;

    let event_row = sqlx::query("SELECT max_players FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    let Some(event_row) = event_row else {
        return Err(not_found("Event not found"));
    };

    let max_players = i32_to_usize(event_row.get::<i32, _>("max_players"), "max_players")?;

    let count_row = sqlx::query("SELECT COUNT(*) AS count FROM event_players WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let current_count = i64_to_usize(count_row.get::<i64, _>("count"), "player count")?;

    if current_count >= max_players {
        return Err(bad_request("Event roster is already full"));
    }

    let player_id = Uuid::new_v4();

    sqlx::query("INSERT INTO event_players (id, event_id, name, role, rank) VALUES ($1, $2, $3, $4, $5)")
        .bind(player_id)
        .bind(event_id)
        .bind(payload.name.trim())
        .bind(payload.role.trim())
        .bind(payload.rank.trim())
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    let event = load_event(&state.pool, event_id).await?;
    Ok(Json(event))
}

pub async fn delete_event_player(
    Path((event_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;

    let exists = sqlx::query("SELECT id FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if exists.is_none() {
        return Err(not_found("Event not found"));
    }

    let deleted = sqlx::query("DELETE FROM event_players WHERE id = $1 AND event_id = $2 RETURNING id")
        .bind(player_id)
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if deleted.is_none() {
        return Err(not_found("Player not found in this event"));
    }

    Ok(Json(MessageResponse {
        message: "Player removed".to_string(),
    }))
}

pub async fn update_event_player(
    Path((event_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventPlayerInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;
    validate_event_player_update_input(&payload)?;

    let updated = sqlx::query(
        "UPDATE event_players SET name = $1, role = $2, rank = $3 WHERE id = $4 AND event_id = $5 RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.role.trim())
    .bind(payload.rank.trim())
    .bind(player_id)
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    if updated.is_none() {
        return Err(not_found("Player not found in this event"));
    }

    let event = load_event(&state.pool, event_id).await?;
    Ok(Json(event))
}

pub async fn create_event_team(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEventTeamInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err(bad_request("Team name is required"));
    }

    let exists = sqlx::query("SELECT id FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if exists.is_none() {
        return Err(not_found("Event not found"));
    }

    let team_id = Uuid::new_v4();

    let inserted = sqlx::query("INSERT INTO event_teams (id, event_id, name) VALUES ($1, $2, $3)")
        .bind(team_id)
        .bind(event_id)
        .bind(name)
        .execute(&state.pool)
        .await;

    if inserted.is_err() {
        return Err(bad_request("Team name already exists in this event"));
    }

    let event = load_event(&state.pool, event_id).await?;
    Ok(Json(event))
}

pub async fn delete_event_team(
    Path((event_id, team_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;

    let exists = sqlx::query("SELECT id FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if exists.is_none() {
        return Err(not_found("Event not found"));
    }

    let deleted = sqlx::query("DELETE FROM event_teams WHERE id = $1 AND event_id = $2 RETURNING id")
        .bind(team_id)
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if deleted.is_none() {
        return Err(not_found("Team not found in this event"));
    }

    sqlx::query(
        "UPDATE event_matches SET team_a_id = NULL WHERE event_id = $1 AND team_a_id = $2",
    )
    .bind(event_id)
    .bind(team_id)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "UPDATE event_matches SET team_b_id = NULL WHERE event_id = $1 AND team_b_id = $2",
    )
    .bind(event_id)
    .bind(team_id)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(MessageResponse {
        message: "Team deleted".to_string(),
    }))
}

pub async fn update_event_team(
    Path((event_id, team_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateEventTeamInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;
    validate_event_team_name(&payload.name)?;

    let updated = sqlx::query("UPDATE event_teams SET name = $1 WHERE id = $2 AND event_id = $3 RETURNING id")
        .bind(payload.name.trim())
        .bind(team_id)
        .bind(event_id)
        .fetch_optional(&state.pool)
        .await;

    let updated = match updated {
        Ok(value) => value,
        Err(_) => return Err(bad_request("Team name already exists in this event")),
    };

    if updated.is_none() {
        return Err(not_found("Team not found in this event"));
    }

    let event = load_event(&state.pool, event_id).await?;
    Ok(Json(event))
}

pub async fn assign_event_player_team(
    Path(event_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AssignEventPlayerTeamInput>,
) -> ApiResult<Event> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;

    let event_id_db = event_id;
    let player_id_db = payload.player_id;

    let player_exists =
        sqlx::query("SELECT id FROM event_players WHERE id = $1 AND event_id = $2")
            .bind(player_id_db)
            .bind(event_id_db)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;

    if player_exists.is_none() {
        return Err(not_found("Player not found in this event"));
    }

    if let Some(team_id) = payload.team_id {
        let team_exists = sqlx::query("SELECT id FROM event_teams WHERE id = $1 AND event_id = $2")
            .bind(team_id)
            .bind(event_id_db)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;

        if team_exists.is_none() {
            return Err(not_found("Team not found in this event"));
        }

        sqlx::query(
            "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (event_id, event_player_id)
             DO UPDATE SET event_team_id = EXCLUDED.event_team_id",
        )
        .bind(Uuid::new_v4())
        .bind(event_id_db)
        .bind(team_id)
        .bind(player_id_db)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    } else {
        sqlx::query("DELETE FROM event_team_members WHERE event_id = $1 AND event_player_id = $2")
            .bind(event_id_db)
            .bind(player_id_db)
            .execute(&state.pool)
            .await
            .map_err(internal_error)?;
    }

    let event = load_event(&state.pool, event_id).await?;
    Ok(Json(event))
}

pub async fn set_matchup(
    Path((event_id, match_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SetMatchupInput>,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;
    require_event_manage_access(&state, event_id, user_id).await?;

    let event_id_db = event_id;
    let match_id_db = match_id;

    let match_exists = sqlx::query("SELECT id FROM event_matches WHERE id = $1 AND event_id = $2")
        .bind(match_id_db)
        .bind(event_id_db)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if match_exists.is_none() {
        return Err(not_found("Match not found in this event"));
    }

    match (payload.team_a_id, payload.team_b_id) {
        (Some(team_a_id), Some(team_b_id)) => {
            if team_a_id == team_b_id {
                return Err(bad_request("A match must have two different teams"));
            }

            let team_a_exists =
                sqlx::query("SELECT id FROM event_teams WHERE id = $1 AND event_id = $2")
                    .bind(team_a_id)
                    .bind(event_id_db)
                    .fetch_optional(&state.pool)
                    .await
                    .map_err(internal_error)?;

            if team_a_exists.is_none() {
                return Err(not_found("Team A not found in this event"));
            }

            let team_b_exists =
                sqlx::query("SELECT id FROM event_teams WHERE id = $1 AND event_id = $2")
                    .bind(team_b_id)
                    .bind(event_id_db)
                    .fetch_optional(&state.pool)
                    .await
                    .map_err(internal_error)?;

            if team_b_exists.is_none() {
                return Err(not_found("Team B not found in this event"));
            }

            sqlx::query("UPDATE event_matches SET team_a_id = $1, team_b_id = $2 WHERE id = $3")
                .bind(team_a_id)
                .bind(team_b_id)
                .bind(match_id_db)
                .execute(&state.pool)
                .await
                .map_err(internal_error)?;
        }
        (None, None) => {
            sqlx::query("UPDATE event_matches SET team_a_id = NULL, team_b_id = NULL WHERE id = $1")
                .bind(match_id_db)
                .execute(&state.pool)
                .await
                .map_err(internal_error)?;
        }
        _ => return Err(bad_request("Provide both teams or clear both")),
    }

    let updated_match = load_match(&state.pool, match_id).await?;
    Ok(Json(updated_match))
}

pub async fn get_match(
    Path(match_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Match> {
    let user_id = require_authenticated_user_id(&state, &headers)?;

    let event_row = sqlx::query("SELECT event_id FROM event_matches WHERE id = $1")
        .bind(match_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    let Some(event_row) = event_row else {
        return Err(not_found("Match not found"));
    };

    let event_id: Uuid = event_row.get("event_id");
    require_event_view_access(&state, event_id, user_id).await?;

    let loaded_match = load_match(&state.pool, match_id).await?;
    Ok(Json(loaded_match))
}

pub async fn delete_match(
    Path(match_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<MessageResponse> {
    let user_id = require_authenticated_user_id(&state, &headers)?;

    let event_row = sqlx::query("SELECT event_id FROM event_matches WHERE id = $1")
        .bind(match_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    let Some(event_row) = event_row else {
        return Err(not_found("Match not found"));
    };

    let event_id: Uuid = event_row.get("event_id");
    require_event_manage_access(&state, event_id, user_id).await?;

    let result = sqlx::query("DELETE FROM event_matches WHERE id = $1")
        .bind(match_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Match not found"));
    }

    Ok(Json(MessageResponse {
        message: "Match deleted".to_string(),
    }))
}

fn validate_create_match_input(payload: &CreateMatchInput) -> Result<(), ApiError> {
    let title = payload.title.trim();
    let map = payload.map.trim();

    if title.is_empty() {
        return Err(bad_request("Match title is required"));
    }

    if map.is_empty() {
        return Err(bad_request("Map is required"));
    }

    if !(2..=12).contains(&payload.max_players) {
        return Err(bad_request("Max players must be between 2 and 12"));
    }

    Ok(())
}

fn validate_create_event_input(payload: &CreateEventInput) -> Result<(), ApiError> {
    let name = payload.name.trim();

    if name.is_empty() {
        return Err(bad_request("Event name is required"));
    }

    if !(2..=12).contains(&payload.max_players) {
        return Err(bad_request("Max players must be between 2 and 12"));
    }

    Ok(())
}

fn validate_update_event_input(payload: &UpdateEventInput) -> Result<(), ApiError> {
    let create_shape = CreateEventInput {
        name: payload.name.clone(),
        event_type: payload.event_type.clone(),
        max_players: payload.max_players,
    };

    validate_create_event_input(&create_shape)
}

fn validate_register_input(payload: &RegisterInput) -> Result<(), ApiError> {
    let email = normalize_email(&payload.email);
    if email.is_empty() || !email.contains('@') {
        return Err(bad_request("A valid email is required"));
    }

    if payload.password.len() < 8 {
        return Err(bad_request("Password must be at least 8 characters long"));
    }

    if payload.display_name.trim().is_empty() {
        return Err(bad_request("Display name is required"));
    }

    Ok(())
}

async fn create_match_record(
    state: &AppState,
    payload: CreateMatchInput,
    event_id: Uuid,
) -> Result<Match, ApiError> {
    validate_create_match_input(&payload)?;

    let match_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO event_matches (id, event_id, title, map, max_players) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(match_id)
    .bind(event_id)
    .bind(payload.title.trim())
    .bind(payload.map.trim())
    .bind(i32::from(payload.max_players))
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    load_match(&state.pool, match_id).await
}

fn validate_add_player_input(payload: &AddPlayerInput) -> Result<(), ApiError> {
    let name = payload.name.trim();
    let role = payload.role.trim();
    let rank = payload.rank.trim();

    if name.is_empty() {
        return Err(bad_request("Player name is required"));
    }

    if role.is_empty() {
        return Err(bad_request("Player role is required"));
    }

    if rank.is_empty() {
        return Err(bad_request("Player rank is required"));
    }

    if !OVERWATCH_RANKS.contains(&rank) {
        return Err(bad_request("Invalid player rank"));
    }

    Ok(())
}

fn validate_event_player_update_input(payload: &UpdateEventPlayerInput) -> Result<(), ApiError> {
    let add_player_shape = AddPlayerInput {
        name: payload.name.clone(),
        role: payload.role.clone(),
        rank: payload.rank.clone(),
    };

    validate_add_player_input(&add_player_shape)
}

fn validate_event_team_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(bad_request("Team name is required"));
    }

    Ok(())
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| bad_request("Failed to hash password"))
}

fn verify_password(stored_hash: &str, password: &str) -> Result<(), ApiError> {
    let parsed_hash = PasswordHash::new(stored_hash).map_err(|_| unauthorized("Invalid email or password"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| unauthorized("Invalid email or password"))
}

async fn issue_auth_response(
    state: &AppState,
    user: AuthUser,
    existing_session_id: Option<Uuid>,
) -> Result<AuthResponse, ApiError> {
    let access_token = build_access_token(user.id, &state.jwt_secret)?;
    let refresh_token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
    let refresh_hash = hash_refresh_token(&refresh_token);

    if let Some(session_id) = existing_session_id {
        sqlx::query(
            "UPDATE auth_sessions
             SET refresh_token_hash = $1,
                 expires_at = NOW() + interval '30 days',
                 revoked_at = NULL
             WHERE id = $2",
        )
        .bind(refresh_hash)
        .bind(session_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    } else {
        sqlx::query(
            "INSERT INTO auth_sessions (id, user_id, refresh_token_hash, expires_at)
             VALUES ($1, $2, $3, NOW() + interval '30 days')",
        )
        .bind(Uuid::new_v4())
        .bind(user.id)
        .bind(refresh_hash)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    }

    Ok(AuthResponse {
        access_token,
        refresh_token,
        user,
    })
}

fn build_access_token(user_id: Uuid, jwt_secret: &str) -> Result<String, ApiError> {
    let now = current_unix_timestamp();
    let claims = AccessClaims {
        sub: user_id.to_string(),
        exp: now + ACCESS_TOKEN_TTL_SECONDS,
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| unauthorized("Failed to generate access token"))
}

fn hash_refresh_token(refresh_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    hex::encode(hasher.finalize())
}

fn current_unix_timestamp() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as usize)
        .unwrap_or(0)
}

fn require_authenticated_user_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| unauthorized("Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| unauthorized("Authorization header must use Bearer token"))?;

    let token_data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| unauthorized("Invalid access token"))?;

    if token_data.claims.token_type != "access" {
        return Err(unauthorized("Invalid access token type"));
    }

    Uuid::parse_str(&token_data.claims.sub).map_err(|_| unauthorized("Invalid access token subject"))
}

async fn get_auth_user_by_id(state: &AppState, user_id: Uuid) -> Result<AuthUser, ApiError> {
    let row = sqlx::query("SELECT id, email, display_name, is_active FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    let Some(row) = row else {
        return Err(unauthorized("User not found"));
    };

    let is_active: bool = row.get("is_active");
    if !is_active {
        return Err(forbidden("User account is inactive"));
    }

    Ok(AuthUser {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
    })
}

async fn require_event_view_access(state: &AppState, event_id: Uuid, user_id: Uuid) -> Result<String, ApiError> {
    let row = sqlx::query("SELECT role FROM event_memberships WHERE event_id = $1 AND user_id = $2")
        .bind(event_id)
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    let Some(row) = row else {
        return Err(forbidden("You do not have access to this event"));
    };

    Ok(row.get("role"))
}

async fn require_event_manage_access(state: &AppState, event_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    let role = require_event_view_access(state, event_id, user_id).await?;
    if role == "owner" || role == "manager" {
        return Ok(());
    }

    Err(forbidden("You do not have permission to modify this event"))
}

async fn require_event_owner_access(state: &AppState, event_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    let role = require_event_view_access(state, event_id, user_id).await?;
    if role == "owner" {
        return Ok(());
    }

    Err(forbidden("Only event owners can perform this action"))
}
