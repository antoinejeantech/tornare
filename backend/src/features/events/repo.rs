use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::shared::errors::{bad_request, internal_error, not_found};
use crate::shared::numeric::i32_to_u8;

use crate::shared::models::{
    Event, EventSignupRequest, EventTeam, EventType, Match, Player, PublicEventSignupInfo,
};

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

pub async fn load_event(pool: &PgPool, event_id: Uuid) -> Result<Event, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT
            e.id,
            e.name,
            e.description,
            e.start_date,
            e.event_type,
            e.max_players,
            u.display_name AS creator_name
         FROM events e
         LEFT JOIN event_memberships m ON m.event_id = e.id AND m.role = 'owner'
         LEFT JOIN users u ON u.id = m.user_id
         WHERE e.id = $1",
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Err(not_found("Event not found"));
    };

    let db_id: Uuid = row.get("id");
    let db_event_type: String = row.get("event_type");
    let event_type = EventType::try_from(db_event_type.as_str())
        .map_err(|_| bad_request("Invalid event type value in database"))?;
    let players = load_event_players_for_event(pool, db_id).await?;
    let teams = load_event_teams_for_event(pool, db_id).await?;
    let matches = load_matches_for_event(pool, db_id).await?;

    Ok(Event {
        id: db_id,
        name: row.get("name"),
        description: row.get("description"),
        start_date: row.get("start_date"),
        event_type,
        is_owner: false,
        creator_name: row.get("creator_name"),
        max_players: i32_to_u8(row.get::<i32, _>("max_players"), "max_players")?,
        players,
        teams,
        matches,
    })
}

pub async fn load_matches_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<Match>, crate::shared::errors::ApiError> {
    let players = load_event_players_for_event(pool, event_id).await?;
    let rows = sqlx::query(
        "SELECT
            g.id,
            g.event_id,
            g.team_a_id,
            ta.name AS team_a_name,
            g.team_b_id,
            tb.name AS team_b_name,
            g.title,
            g.map,
            g.max_players
         FROM event_matches g
         LEFT JOIN event_teams ta ON ta.id = g.team_a_id
         LEFT JOIN event_teams tb ON tb.id = g.team_b_id
         WHERE g.event_id = $1
         ORDER BY g.id ASC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let mut matches = Vec::with_capacity(rows.len());
    for row in rows {
        let match_id: Uuid = row.get("id");

        matches.push(Match {
            id: match_id,
            event_id: row.get::<Option<Uuid>, _>("event_id"),
            team_a_id: row.get::<Option<Uuid>, _>("team_a_id"),
            team_a_name: row.get("team_a_name"),
            team_b_id: row.get::<Option<Uuid>, _>("team_b_id"),
            team_b_name: row.get("team_b_name"),
            title: row.get("title"),
            map: row.get("map"),
            max_players: i32_to_u8(row.get::<i32, _>("max_players"), "max_players")?,
            players: players.clone(),
        });
    }

    Ok(matches)
}

pub async fn load_event_players_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<Player>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT
            ep.id,
            ep.name,
            ep.role,
            ep.rank,
            et.id AS team_id,
            et.name AS team_name
         FROM event_players ep
         LEFT JOIN event_team_members etm ON etm.event_player_id = ep.id
         LEFT JOIN event_teams et ON et.id = etm.event_team_id
         WHERE ep.event_id = $1
         ORDER BY ep.id ASC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let mut players = Vec::with_capacity(rows.len());
    for row in rows {
        players.push(Player {
            id: row.get::<Uuid, _>("id"),
            name: row.get("name"),
            role: row.get("role"),
            rank: row.get("rank"),
            team_id: row.get::<Option<Uuid>, _>("team_id"),
            team: row.get("team_name"),
        });
    }

    Ok(players)
}

pub async fn load_event_teams_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<EventTeam>, crate::shared::errors::ApiError> {
    let team_rows =
        sqlx::query("SELECT id, name FROM event_teams WHERE event_id = $1 ORDER BY id ASC")
            .bind(event_id)
            .fetch_all(pool)
            .await
            .map_err(internal_error)?;

    let mut teams = Vec::with_capacity(team_rows.len());
    for row in team_rows {
        let team_id = row.get::<Uuid, _>("id");
        let member_rows = sqlx::query(
            "SELECT event_player_id FROM event_team_members WHERE event_id = $1 AND event_team_id = $2 ORDER BY event_player_id ASC",
        )
        .bind(event_id)
        .bind(team_id)
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;

        let mut player_ids = Vec::with_capacity(member_rows.len());
        for member in member_rows {
            player_ids.push(member.get::<Uuid, _>("event_player_id"));
        }

        teams.push(EventTeam {
            id: team_id,
            name: row.get("name"),
            player_ids,
        });
    }

    Ok(teams)
}
