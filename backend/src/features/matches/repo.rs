use sqlx::{PgPool, Row, Transaction};
use uuid::Uuid;

use crate::shared::{
    errors::{internal_error, not_found},
    numeric::i32_to_u8,
};
use crate::features::events::models::{Match, Player};

pub async fn list_visible_match_ids(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Uuid>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT em.id
         FROM event_matches em
         INNER JOIN event_memberships m ON m.event_id = em.event_id
         WHERE m.user_id = $1
         ORDER BY em.id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

pub async fn get_match_event_id(
    pool: &PgPool,
    match_id: Uuid,
) -> Result<Option<Uuid>, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT event_id FROM event_matches WHERE id = $1")
        .bind(match_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.map(|r| r.get("event_id")))
}

pub async fn delete_match_by_id(
    pool: &PgPool,
    match_id: Uuid,
) -> Result<u64, crate::shared::errors::ApiError> {
    let result = sqlx::query("DELETE FROM event_matches WHERE id = $1")
        .bind(match_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(result.rows_affected())
}

pub async fn count_event_matches(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<i64, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT COUNT(*) AS count FROM event_matches WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.get("count"))
}

pub async fn insert_event_match(
    pool: &PgPool,
    match_id: Uuid,
    event_id: Uuid,
    title: &str,
    map: &str,
    max_players: i32,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("INSERT INTO event_matches (id, event_id, title, map, max_players) VALUES ($1, $2, $3, $4, $5)")
        .bind(match_id)
        .bind(event_id)
        .bind(title)
        .bind(map)
        .bind(max_players)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub struct BracketMatchInsert<'a> {
    pub id: Uuid,
    pub event_id: Uuid,
    pub team_a_id: Option<Uuid>,
    pub team_b_id: Option<Uuid>,
    pub title: &'a str,
    pub map: &'a str,
    pub max_players: i32,
    pub round: i32,
    pub position: i32,
    pub winner_team_id: Option<Uuid>,
    pub status: &'a str,
}

pub async fn insert_bracket_match_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    input: BracketMatchInsert<'_>,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_matches (
            id, event_id, team_a_id, team_b_id, title, map, max_players,
            round, position, next_match_id, next_match_slot, winner_team_id,
            is_bracket, status
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12,
            TRUE, $13
         )",
    )
    .bind(input.id)
    .bind(input.event_id)
    .bind(input.team_a_id)
    .bind(input.team_b_id)
    .bind(input.title)
    .bind(input.map)
    .bind(input.max_players)
    .bind(input.round)
    .bind(input.position)
    .bind(Option::<Uuid>::None)
    .bind(Option::<&str>::None)
    .bind(input.winner_team_id)
    .bind(input.status)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn update_bracket_next_link_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
    event_id: Uuid,
    next_match_id: Option<Uuid>,
    next_match_slot: Option<&str>,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE event_matches
         SET next_match_id = $1,
             next_match_slot = $2
         WHERE id = $3 AND event_id = $4",
    )
    .bind(next_match_id)
    .bind(next_match_slot)
    .bind(match_id)
    .bind(event_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn normalize_bracket_matches_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE event_matches
         SET winner_team_id = NULL
         WHERE event_id = $1
           AND is_bracket = TRUE
           AND winner_team_id IS NOT NULL
           AND (team_a_id IS NULL OR team_b_id IS NULL)",
    )
    .bind(event_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "UPDATE event_matches
         SET status = CASE
             WHEN winner_team_id IS NOT NULL THEN 'COMPLETED'
             WHEN team_a_id IS NOT NULL AND team_b_id IS NOT NULL THEN 'READY'
             ELSE 'OPEN'
         END
         WHERE event_id = $1
           AND is_bracket = TRUE",
    )
    .bind(event_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub struct BracketMatchState {
    pub team_a_id: Option<Uuid>,
    pub team_b_id: Option<Uuid>,
    pub winner_team_id: Option<Uuid>,
    pub is_bracket: bool,
}

pub async fn get_bracket_match_state_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
    match_id: Uuid,
) -> Result<Option<BracketMatchState>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT
            team_a_id,
            team_b_id,
            winner_team_id,
            is_bracket
         FROM event_matches
         WHERE id = $1 AND event_id = $2",
    )
    .bind(match_id)
    .bind(event_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(row.map(|row| BracketMatchState {
        team_a_id: row.get("team_a_id"),
        team_b_id: row.get("team_b_id"),
        winner_team_id: row.get("winner_team_id"),
        is_bracket: row.get("is_bracket"),
    }))
}

pub async fn list_bracket_match_ids_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
) -> Result<Vec<Uuid>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT id
         FROM event_matches
         WHERE event_id = $1 AND is_bracket = TRUE
         ORDER BY round ASC NULLS LAST, position ASC NULLS LAST, id ASC",
    )
    .bind(event_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

pub async fn set_match_winner_completed_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
    winner_team_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE event_matches SET winner_team_id = $1, status = 'COMPLETED' WHERE id = $2")
        .bind(winner_team_id)
        .bind(match_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn get_next_match_link_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
) -> Result<Option<(Uuid, Option<String>)>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT next_match_id, next_match_slot
         FROM event_matches
         WHERE id = $1",
    )
    .bind(match_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Ok(None);
    };

    let next_match_id: Option<Uuid> = row.get("next_match_id");
    Ok(next_match_id.map(|id| (id, row.get("next_match_slot"))))
}

pub async fn set_matchup_slot_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
    slot: &str,
    team_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    match slot {
        "A" => {
            sqlx::query("UPDATE event_matches SET team_a_id = $1 WHERE id = $2")
                .bind(team_id)
                .bind(match_id)
                .execute(&mut **tx)
                .await
                .map_err(internal_error)?;
        }
        "B" => {
            sqlx::query("UPDATE event_matches SET team_b_id = $1 WHERE id = $2")
                .bind(team_id)
                .bind(match_id)
                .execute(&mut **tx)
                .await
                .map_err(internal_error)?;
        }
        _ => {}
    }

    Ok(())
}

pub async fn get_match_state_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
) -> Result<(Option<Uuid>, Option<Uuid>, Option<Uuid>), crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT team_a_id, team_b_id, winner_team_id
         FROM event_matches
         WHERE id = $1",
    )
    .bind(match_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok((
        row.get("team_a_id"),
        row.get("team_b_id"),
        row.get("winner_team_id"),
    ))
}

pub async fn set_match_status_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
    status: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE event_matches SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(match_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn set_matchup(
    pool: &PgPool,
    match_id: Uuid,
    team_a_id: Uuid,
    team_b_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE event_matches SET team_a_id = $1, team_b_id = $2 WHERE id = $3")
        .bind(team_a_id)
        .bind(team_b_id)
        .bind(match_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn clear_matchup(
    pool: &PgPool,
    match_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("UPDATE event_matches SET team_a_id = NULL, team_b_id = NULL WHERE id = $1")
        .bind(match_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn load_match(pool: &PgPool, match_id: Uuid) -> Result<Match, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT
            g.id,
            g.event_id,
            g.team_a_id,
            ta.name AS team_a_name,
            g.team_b_id,
            tb.name AS team_b_name,
            g.title,
            g.map,
                g.max_players,
                g.round,
                g.position,
                g.next_match_id,
                g.next_match_slot,
                g.winner_team_id,
                tw.name AS winner_team_name,
                g.is_bracket,
                g.status
         FROM event_matches g
         LEFT JOIN event_teams ta ON ta.id = g.team_a_id
         LEFT JOIN event_teams tb ON tb.id = g.team_b_id
            LEFT JOIN event_teams tw ON tw.id = g.winner_team_id
         WHERE g.id = $1",
    )
    .bind(match_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Err(not_found("Match not found"));
    };

    let db_id: Uuid = row.get("id");
    let event_id: Uuid = row.get("event_id");
    let players = load_event_players_for_event(pool, event_id).await?;

    Ok(Match {
        id: db_id,
        event_id: Some(event_id),
        team_a_id: row.get::<Option<Uuid>, _>("team_a_id"),
        team_a_name: row.get("team_a_name"),
        team_b_id: row.get::<Option<Uuid>, _>("team_b_id"),
        team_b_name: row.get("team_b_name"),
        title: row.get("title"),
        map: row.get("map"),
        max_players: i32_to_u8(row.get::<i32, _>("max_players"), "max_players")?,
        round: row.get::<Option<i32>, _>("round"),
        position: row.get::<Option<i32>, _>("position"),
        next_match_id: row.get::<Option<Uuid>, _>("next_match_id"),
        next_match_slot: row.get::<Option<String>, _>("next_match_slot"),
        winner_team_id: row.get::<Option<Uuid>, _>("winner_team_id"),
        winner_team_name: row.get("winner_team_name"),
        is_bracket: row.get::<bool, _>("is_bracket"),
        status: row.get::<String, _>("status"),
        players,
    })
}

async fn load_event_players_for_event(
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
