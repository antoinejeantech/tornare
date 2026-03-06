use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::shared::errors::{bad_request, internal_error};

use crate::shared::models::{EventSignupRequest, EventType, PublicEventSignupInfo};

pub async fn list_visible_event_ids(
    pool: &PgPool,
) -> Result<Vec<Uuid>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT e.id
         FROM events e
         ORDER BY e.id DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

pub async fn event_exists(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;
    Ok(row.is_some())
}

pub async fn event_max_players(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Option<i32>, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT max_players FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.map(|r| r.get("max_players")))
}

pub async fn count_event_players(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<i64, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT COUNT(*) AS count FROM event_players WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.get("count"))
}

pub async fn event_player_exists(
    pool: &PgPool,
    event_id: Uuid,
    player_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM event_players WHERE id = $1 AND event_id = $2")
        .bind(player_id)
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;
    Ok(row.is_some())
}

pub async fn event_team_exists(
    pool: &PgPool,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM event_teams WHERE id = $1 AND event_id = $2")
        .bind(team_id)
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;
    Ok(row.is_some())
}

pub async fn event_match_exists(
    pool: &PgPool,
    event_id: Uuid,
    match_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT id FROM event_matches WHERE id = $1 AND event_id = $2")
        .bind(match_id)
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;
    Ok(row.is_some())
}

pub async fn is_event_owner(
    pool: &PgPool,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id
             FROM event_memberships
             WHERE event_id = $1 AND user_id = $2 AND role = 'owner'",
    )
    .bind(event_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn signup_token_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Option<String>, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT signup_token FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.map(|value| value.get("signup_token")))
}

pub async fn rotate_signup_token_for_event(
    pool: &PgPool,
    event_id: Uuid,
    signup_token: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let result = sqlx::query("UPDATE events SET signup_token = $1 WHERE id = $2")
        .bind(signup_token)
        .bind(event_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(result.rows_affected() > 0)
}

pub async fn event_signup_info_by_token(
    pool: &PgPool,
    signup_token: &str,
) -> Result<Option<PublicEventSignupInfo>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT
                e.id,
                e.name,
                e.event_type,
                e.max_players,
                COUNT(ep.id) AS current_players
             FROM events e
             LEFT JOIN event_players ep ON ep.event_id = e.id
             WHERE e.signup_token = $1
             GROUP BY e.id, e.name, e.event_type, e.max_players",
    )
    .bind(signup_token)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Ok(None);
    };

    let event_type_db: String = row.get("event_type");
    let event_type = EventType::try_from(event_type_db.as_str())
        .map_err(|_| bad_request("Invalid event type value in database"))?;

    let current_players_i64: i64 = row.get("current_players");
    let current_players = usize::try_from(current_players_i64)
        .map_err(|_| bad_request("Invalid current players value in database"))?;

    let max_players = u8::try_from(row.get::<i32, _>("max_players"))
        .map_err(|_| bad_request("Invalid max players value in database"))?;

    Ok(Some(PublicEventSignupInfo {
        event_id: row.get("id"),
        event_name: row.get("name"),
        event_type,
        max_players,
        current_players,
    }))
}

pub async fn create_signup_request(
    pool: &PgPool,
    event_id: Uuid,
    name: &str,
    role: &str,
    rank: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_signup_requests (id, event_id, name, role, rank, status)
             VALUES ($1, $2, $3, $4, $5, 'pending')",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(name)
    .bind(role)
    .bind(rank)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn has_pending_signup_request_with_name(
    pool: &PgPool,
    event_id: Uuid,
    name: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id
             FROM event_signup_requests
             WHERE event_id = $1
               AND status = 'pending'
               AND LOWER(name) = LOWER($2)
             LIMIT 1",
    )
    .bind(event_id)
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn list_signup_requests_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<EventSignupRequest>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT id, event_id, name, role, rank, status
             FROM event_signup_requests
             WHERE event_id = $1
             ORDER BY created_at DESC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows
        .into_iter()
        .map(|row| EventSignupRequest {
            id: row.get("id"),
            event_id: row.get("event_id"),
            name: row.get("name"),
            role: row.get("role"),
            rank: row.get("rank"),
            status: row.get("status"),
        })
        .collect())
}

pub async fn get_signup_request(
    pool: &PgPool,
    event_id: Uuid,
    request_id: Uuid,
) -> Result<Option<EventSignupRequest>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id, event_id, name, role, rank, status
             FROM event_signup_requests
             WHERE event_id = $1 AND id = $2",
    )
    .bind(event_id)
    .bind(request_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|value| EventSignupRequest {
        id: value.get("id"),
        event_id: value.get("event_id"),
        name: value.get("name"),
        role: value.get("role"),
        rank: value.get("rank"),
        status: value.get("status"),
    }))
}

pub async fn update_signup_request_status(
    pool: &PgPool,
    event_id: Uuid,
    request_id: Uuid,
    status: &str,
) -> Result<u64, crate::shared::errors::ApiError> {
    let result = sqlx::query(
        "UPDATE event_signup_requests
             SET status = $1
             WHERE event_id = $2 AND id = $3 AND status = 'pending'",
    )
    .bind(status)
    .bind(event_id)
    .bind(request_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(result.rows_affected())
}
