use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::shared::{
    errors::{internal_error, not_found},
    models::{Match, Player},
    numeric::i32_to_u8,
};

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
