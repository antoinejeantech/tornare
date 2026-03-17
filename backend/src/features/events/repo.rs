use sqlx::{Postgres, QueryBuilder, PgPool, Row, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::shared::errors::{bad_request, internal_error, not_found};
use crate::shared::numeric::i32_to_u8;

use crate::features::events::models::{
    Event, EventFormat, EventSignupRequest, EventTeam, EventType, Match, Player,
    PublicEventSignupInfo,
};

#[derive(Clone, Copy)]
pub enum EventListSort {
    Soonest,
    Newest,
    Players,
    Name,
}

pub struct ListVisibleEventsOptions {
    pub search: Option<String>,
    pub event_type: Option<String>,
    pub owner_only_user_id: Option<Uuid>,
    pub sort: EventListSort,
    pub limit: u32,
    pub offset: u32,
}

pub struct ListVisibleEventsResult {
    pub event_ids: Vec<Uuid>,
    pub total: u64,
}

pub struct EventsKpiRow {
    pub total_events: i64,
    pub total_signups: i64,
    pub upcoming_events_this_week: i64,
    pub upcoming_tourneys_this_week: i64,
}

pub async fn load_events_kpis(
    pool: &PgPool,
) -> Result<EventsKpiRow, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT
            (SELECT COUNT(*) FROM events) AS total_events,
            (SELECT COUNT(*) FROM event_players) AS total_signups,
            (
                SELECT COUNT(*)
                FROM events e
                WHERE e.start_date IS NOT NULL
                                    AND e.start_date >= NOW()
                                    AND e.start_date <= NOW() + INTERVAL '7 days'
            ) AS upcoming_events_this_week,
            (
                SELECT COUNT(*)
                FROM events e
                WHERE e.event_type = 'TOURNEY'
                  AND e.start_date IS NOT NULL
                                    AND e.start_date >= NOW()
                                    AND e.start_date <= NOW() + INTERVAL '7 days'
            ) AS upcoming_tourneys_this_week",
    )
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    Ok(EventsKpiRow {
        total_events: row.get("total_events"),
        total_signups: row.get("total_signups"),
        upcoming_events_this_week: row.get("upcoming_events_this_week"),
        upcoming_tourneys_this_week: row.get("upcoming_tourneys_this_week"),
    })
}

pub async fn featured_event_id(
    pool: &PgPool,
) -> Result<Option<Uuid>, crate::shared::errors::ApiError> {
    let featured = sqlx::query(
        "SELECT id
         FROM events
         WHERE is_featured = TRUE
         ORDER BY start_date IS NULL, start_date ASC, id DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    if let Some(row) = featured {
        return Ok(Some(row.get("id")));
    }

    let upcoming = sqlx::query(
        "SELECT id
         FROM events
         WHERE start_date IS NOT NULL
                     AND start_date >= NOW()
         ORDER BY start_date ASC, id DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    if let Some(row) = upcoming {
        return Ok(Some(row.get("id")));
    }

    let fallback = sqlx::query(
        "SELECT id
         FROM events
         ORDER BY start_date IS NULL, start_date ASC, id DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(fallback.map(|row| row.get("id")))
}

pub async fn list_visible_event_ids(
    pool: &PgPool,
    options: ListVisibleEventsOptions,
) -> Result<ListVisibleEventsResult, crate::shared::errors::ApiError> {
    let mut count_query_builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "SELECT COUNT(*) AS total
         FROM events e
         WHERE 1=1",
    );

    apply_event_list_filters(&mut count_query_builder, &options);

    let total_row = count_query_builder
        .build()
        .fetch_one(pool)
        .await
        .map_err(internal_error)?;
    let total_i64: i64 = total_row.get("total");
    let total = if total_i64 < 0 { 0 } else { total_i64 as u64 };

    let mut query_builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "SELECT e.id
         FROM events e
         WHERE 1=1",
    );

    apply_event_list_filters(&mut query_builder, &options);

    match options.sort {
        EventListSort::Newest => {
            query_builder.push(" ORDER BY e.start_date IS NULL, e.start_date DESC, e.id DESC");
        }
        EventListSort::Players => {
            query_builder.push(
                " ORDER BY (
                    SELECT COUNT(*)
                    FROM event_players ep
                    WHERE ep.event_id = e.id
                ) DESC, e.start_date IS NULL, e.start_date ASC, e.id DESC",
            );
        }
        EventListSort::Name => {
            query_builder.push(" ORDER BY LOWER(e.name) ASC, e.id DESC");
        }
        EventListSort::Soonest => {
            query_builder.push(" ORDER BY e.start_date IS NULL, e.start_date ASC, e.id DESC");
        }
    }

    query_builder.push(" LIMIT ");
    query_builder.push_bind(options.limit as i64);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(options.offset as i64);

    let rows = query_builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;

    Ok(ListVisibleEventsResult {
        event_ids: rows.into_iter().map(|row| row.get("id")).collect(),
        total,
    })
}

fn apply_event_list_filters(
    query_builder: &mut QueryBuilder<'_, Postgres>,
    options: &ListVisibleEventsOptions,
) {
    if let Some(event_type) = options.event_type.as_ref() {
        query_builder.push(" AND e.event_type = ");
        query_builder.push_bind(event_type.clone());
    }

    if let Some(owner_user_id) = options.owner_only_user_id {
        query_builder.push(
            " AND EXISTS (
                SELECT 1
                FROM event_memberships em
                WHERE em.event_id = e.id
                  AND em.user_id = ",
        );
        query_builder.push_bind(owner_user_id);
        query_builder.push(" AND em.role = 'owner')");
    }

    if let Some(search) = options.search.as_ref() {
        let like_pattern = format!("%{search}%");
        query_builder.push(" AND (");
        query_builder.push("e.name ILIKE ");
        query_builder.push_bind(like_pattern.clone());
        query_builder.push(" OR e.description ILIKE ");
        query_builder.push_bind(like_pattern.clone());
        query_builder.push(
            " OR EXISTS (
                SELECT 1
                FROM event_memberships ems
                INNER JOIN users u ON u.id = ems.user_id
                WHERE ems.event_id = e.id
                  AND ems.role = 'owner'
                  AND (
                    COALESCE(u.username, '') ILIKE ",
        );
        query_builder.push_bind(like_pattern.clone());
        query_builder.push(" OR COALESCE(u.display_name, '') ILIKE ");
        query_builder.push_bind(like_pattern);
        query_builder.push(")");
        query_builder.push(")");
        query_builder.push(")");
    }
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

pub async fn insert_event(
    pool: &PgPool,
    event_id: Uuid,
    name: &str,
    description: &str,
    start_date: Option<OffsetDateTime>,
    event_type: &str,
    format: &str,
    public_signup_enabled: bool,
    max_players: i32,
    signup_token: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO events (id, name, description, start_date, event_type, format, public_signup_enabled, max_players, signup_token)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(event_id)
    .bind(name)
    .bind(description)
    .bind(start_date)
    .bind(event_type)
    .bind(format)
    .bind(public_signup_enabled)
    .bind(max_players)
    .bind(signup_token)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn insert_event_owner_membership(
    pool: &PgPool,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_memberships (id, event_id, user_id, role) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(user_id)
    .bind("owner")
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn update_event_details(
    pool: &PgPool,
    event_id: Uuid,
    name: &str,
    description: &str,
    start_date: Option<OffsetDateTime>,
    event_type: &str,
    format: &str,
    max_players: i32,
) -> Result<bool, crate::shared::errors::ApiError> {
    let updated = sqlx::query(
        "UPDATE events
         SET name = $1, description = $2, start_date = $3, event_type = $4, format = $5, max_players = $6
         WHERE id = $7
         RETURNING id",
    )
    .bind(name)
    .bind(description)
    .bind(start_date)
    .bind(event_type)
    .bind(format)
    .bind(max_players)
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(updated.is_some())
}

pub async fn delete_event_by_id(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<u64, crate::shared::errors::ApiError> {
    let result = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(event_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(result.rows_affected())
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

pub async fn event_type_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Option<EventType>, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT event_type FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    let Some(row) = row else {
        return Ok(None);
    };

    let event_type_db: String = row.get("event_type");
    let event_type = EventType::try_from(event_type_db.as_str())
        .map_err(|_| bad_request("Invalid event type value in database"))?;

    Ok(Some(event_type))
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

pub async fn insert_event_player(
    pool: &PgPool,
    event_id: Uuid,
    name: &str,
    role: &str,
    rank: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_players (id, event_id, name, role, rank) VALUES ($1, $2, $3, $4, $5)",
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

pub async fn delete_event_player_by_id(
    pool: &PgPool,
    event_id: Uuid,
    player_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let deleted =
        sqlx::query("DELETE FROM event_players WHERE id = $1 AND event_id = $2 RETURNING id")
            .bind(player_id)
            .bind(event_id)
            .fetch_optional(pool)
            .await
            .map_err(internal_error)?;

    Ok(deleted.is_some())
}

pub async fn update_event_player_by_id(
    pool: &PgPool,
    event_id: Uuid,
    player_id: Uuid,
    name: &str,
    role: &str,
    rank: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let updated = sqlx::query(
        "UPDATE event_players SET name = $1, role = $2, rank = $3 WHERE id = $4 AND event_id = $5 RETURNING id",
    )
    .bind(name)
    .bind(role)
    .bind(rank)
    .bind(player_id)
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(updated.is_some())
}

pub async fn upsert_event_player_team_membership(
    pool: &PgPool,
    event_id: Uuid,
    team_id: Uuid,
    player_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (event_id, event_player_id)
         DO UPDATE SET event_team_id = EXCLUDED.event_team_id",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(team_id)
    .bind(player_id)
    .execute(pool)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn delete_event_player_team_membership(
    pool: &PgPool,
    event_id: Uuid,
    player_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("DELETE FROM event_team_members WHERE event_id = $1 AND event_player_id = $2")
        .bind(event_id)
        .bind(player_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
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

pub async fn insert_event_team(
    pool: &PgPool,
    event_id: Uuid,
    team_name: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let inserted = sqlx::query("INSERT INTO event_teams (id, event_id, name) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(team_name)
        .execute(pool)
        .await;

    Ok(inserted.is_ok())
}

pub async fn delete_event_team_by_id(
    pool: &PgPool,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let deleted =
        sqlx::query("DELETE FROM event_teams WHERE id = $1 AND event_id = $2 RETURNING id")
            .bind(team_id)
            .bind(event_id)
            .fetch_optional(pool)
            .await
            .map_err(internal_error)?;

    Ok(deleted.is_some())
}

pub async fn clear_team_from_event_matches(
    pool: &PgPool,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE event_matches
         SET team_a_id = NULL,
             updated_at = NOW()
         WHERE event_id = $1 AND team_a_id = $2",
    )
        .bind(event_id)
        .bind(team_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    sqlx::query(
        "UPDATE event_matches
         SET team_b_id = NULL,
             updated_at = NOW()
         WHERE event_id = $1 AND team_b_id = $2",
    )
        .bind(event_id)
        .bind(team_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn count_played_matches_for_team(
    pool: &PgPool,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<i64, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT COUNT(*) AS count
         FROM event_matches
         WHERE event_id = $1
           AND winner_team_id IS NOT NULL
           AND (
             team_a_id = $2
             OR team_b_id = $2
             OR winner_team_id = $2
           )",
    )
    .bind(event_id)
    .bind(team_id)
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.get("count"))
}

pub async fn update_event_team_name_by_id(
    pool: &PgPool,
    event_id: Uuid,
    team_id: Uuid,
    team_name: &str,
) -> Result<TeamNameUpdateOutcome, crate::shared::errors::ApiError> {
    let updated = sqlx::query(
        "UPDATE event_teams SET name = $1 WHERE id = $2 AND event_id = $3 RETURNING id",
    )
    .bind(team_name)
    .bind(team_id)
    .bind(event_id)
    .fetch_optional(pool)
    .await;

    match updated {
        Ok(value) => {
            if value.is_some() {
                Ok(TeamNameUpdateOutcome::Updated)
            } else {
                Ok(TeamNameUpdateOutcome::NotFound)
            }
        }
        Err(_) => Ok(TeamNameUpdateOutcome::DuplicateName),
    }
}

pub enum TeamNameUpdateOutcome {
    Updated,
    NotFound,
    DuplicateName,
}

pub async fn event_has_matches(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM event_matches WHERE event_id = $1) AS has_matches")
        .bind(event_id)
        .fetch_one(pool)
        .await
        .map_err(internal_error)?;

    Ok(row.get("has_matches"))
}

pub struct UnassignedEventPlayer {
    pub id: Uuid,
    pub name: String,
}

pub async fn list_unassigned_event_players(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<UnassignedEventPlayer>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT ep.id, ep.name
         FROM event_players ep
         LEFT JOIN event_team_members etm ON etm.event_id = ep.event_id AND etm.event_player_id = ep.id
         WHERE ep.event_id = $1 AND etm.id IS NULL
         ORDER BY ep.name ASC, ep.id ASC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    Ok(rows
        .into_iter()
        .map(|row| UnassignedEventPlayer {
            id: row.get("id"),
            name: row.get("name"),
        })
        .collect())
}

pub async fn list_event_team_names(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<String>, crate::shared::errors::ApiError> {
    let rows = sqlx::query("SELECT name FROM event_teams WHERE event_id = $1")
        .bind(event_id)
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;

    Ok(rows.into_iter().map(|row| row.get("name")).collect())
}

pub async fn insert_event_team_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
    team_id: Uuid,
    team_name: &str,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("INSERT INTO event_teams (id, event_id, name) VALUES ($1, $2, $3)")
        .bind(team_id)
        .bind(event_id)
        .bind(team_name)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn insert_event_team_membership_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
    team_id: Uuid,
    player_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(team_id)
    .bind(player_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn clear_event_team_memberships_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("DELETE FROM event_team_members WHERE event_id = $1")
        .bind(event_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;

    Ok(())
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
            e.description,
                e.start_date,
                e.event_type,
                e.format,
                e.max_players,
                (
                    SELECT COUNT(*)
                    FROM event_players ep
                    WHERE ep.event_id = e.id
                ) AS current_players,
                (
                    SELECT COUNT(*)
                    FROM event_signup_requests sr
                    WHERE sr.event_id = e.id
                      AND sr.status = 'pending'
                ) AS current_signup_requests
             FROM events e
                         WHERE e.signup_token = $1",
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
    let format_db: String = row.get("format");
    let format = EventFormat::try_from(format_db.as_str())
        .map_err(|_| bad_request("Invalid event format value in database"))?;

    let current_players_i64: i64 = row.get("current_players");
    let current_players = usize::try_from(current_players_i64)
        .map_err(|_| bad_request("Invalid current players value in database"))?;

    let current_signup_requests_i64: i64 = row.get("current_signup_requests");
    let current_signup_requests = usize::try_from(current_signup_requests_i64)
        .map_err(|_| bad_request("Invalid current signup requests value in database"))?;

    let max_players = u8::try_from(row.get::<i32, _>("max_players"))
        .map_err(|_| bad_request("Invalid max players value in database"))?;

    Ok(Some(PublicEventSignupInfo {
        event_id: row.get("id"),
        event_name: row.get("name"),
        event_description: row.get("description"),
        start_date: row.get::<Option<OffsetDateTime>, _>("start_date"),
        event_type,
        format,
        max_players,
        current_players,
        current_signup_requests,
    }))
}

pub async fn set_public_signup_enabled_for_event(
    pool: &PgPool,
    event_id: Uuid,
    enabled: bool,
    signup_token: Option<&str>,
) -> Result<bool, crate::shared::errors::ApiError> {
    let updated = sqlx::query(
        "UPDATE events
         SET public_signup_enabled = $1,
             signup_token = COALESCE($2, signup_token)
         WHERE id = $3
         RETURNING id",
    )
    .bind(enabled)
    .bind(signup_token)
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(updated.is_some())
}

pub async fn set_featured_event_state(
    pool: &PgPool,
    event_id: Uuid,
    featured: bool,
) -> Result<(), crate::shared::errors::ApiError> {
    let mut tx = pool.begin().await.map_err(internal_error)?;

    if featured {
        sqlx::query("UPDATE events SET is_featured = FALSE WHERE is_featured = TRUE AND id <> $1")
            .bind(event_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;

        sqlx::query("UPDATE events SET is_featured = TRUE WHERE id = $1")
            .bind(event_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    } else {
        sqlx::query("UPDATE events SET is_featured = FALSE WHERE id = $1")
            .bind(event_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    Ok(())
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
            e.format,
            e.is_featured,
            e.signup_token,
            e.public_signup_enabled,
            e.max_players,
            m.user_id AS creator_id,
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
    let db_format: String = row.get("format");
    let format = EventFormat::try_from(db_format.as_str())
        .map_err(|_| bad_request("Invalid event format value in database"))?;
    let players = load_event_players_for_event(pool, db_id).await?;
    let teams = load_event_teams_for_event(pool, db_id).await?;
    let matches = load_matches_for_event(pool, db_id).await?;

    Ok(Event {
        id: db_id,
        name: row.get("name"),
        description: row.get("description"),
        start_date: row.get::<Option<OffsetDateTime>, _>("start_date"),
        event_type,
        format,
        is_featured: row.get("is_featured"),
        is_owner: false,
        can_manage: false,
        creator_id: row.get("creator_id"),
        creator_name: row.get("creator_name"),
        public_signup_enabled: row.get("public_signup_enabled"),
        public_signup_token: {
            let is_public: bool = row.get("public_signup_enabled");
            if is_public {
                row.get("signup_token")
            } else {
                None
            }
        },
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
                g.max_players,
                g.round,
                g.position,
                g.next_match_id,
                g.next_match_slot,
                g.winner_team_id,
                tw.name AS winner_team_name,
                g.is_bracket,
                g.status,
                  g.created_at,
                  g.updated_at,
                  g.start_date
         FROM event_matches g
         LEFT JOIN event_teams ta ON ta.id = g.team_a_id
         LEFT JOIN event_teams tb ON tb.id = g.team_b_id
            LEFT JOIN event_teams tw ON tw.id = g.winner_team_id
         WHERE g.event_id = $1
                ORDER BY COALESCE(g.round, 9999), COALESCE(g.position, 9999), g.created_at ASC, g.id ASC",
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
            round: row.get::<Option<i32>, _>("round"),
            position: row.get::<Option<i32>, _>("position"),
            next_match_id: row.get::<Option<Uuid>, _>("next_match_id"),
            next_match_slot: row.get::<Option<String>, _>("next_match_slot"),
            winner_team_id: row.get::<Option<Uuid>, _>("winner_team_id"),
            winner_team_name: row.get("winner_team_name"),
            is_bracket: row.get::<bool, _>("is_bracket"),
            status: row.get::<String, _>("status"),
            created_at: row.get::<OffsetDateTime, _>("created_at"),
            updated_at: row.get::<OffsetDateTime, _>("updated_at"),
            start_date: row.get::<Option<OffsetDateTime>, _>("start_date"),
            players: players.clone(),
        });
    }

    Ok(matches)
}

pub async fn list_team_ids_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<Uuid>, crate::shared::errors::ApiError> {
    let rows = sqlx::query("SELECT id FROM event_teams WHERE event_id = $1 ORDER BY id ASC")
        .bind(event_id)
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
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
