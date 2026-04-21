use sqlx::{Postgres, QueryBuilder, PgPool, Row, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::shared::errors::{internal_error, not_found};
use crate::shared::numeric::i32_to_u8;

use crate::features::events::models::{
    Event, EventFormat, EventSignupRequest, EventStatus, EventTeam, EventType, LinkedUserInfo, Match, MatchStatus, Player,
    PlayerRank, PlayerRole, PublicEventSignupInfo, RolePreference, RolePreferenceInput, SignupStatus,
};

fn parse_player_rank_db(rank_str: &str) -> Result<PlayerRank, crate::shared::errors::ApiError> {
    PlayerRank::try_from(rank_str)
        .map_err(|_| internal_error(format!("invalid player rank in DB: {rank_str}")))
}

fn expand_db_role_preferences(
    role_str: &str,
    rank: PlayerRank,
) -> Result<Vec<RolePreference>, crate::shared::errors::ApiError> {
    match role_str {
        "Tank" => Ok(vec![RolePreference {
            role: PlayerRole::Tank,
            rank,
        }]),
        "DPS" => Ok(vec![RolePreference {
            role: PlayerRole::Dps,
            rank,
        }]),
        "Support" => Ok(vec![RolePreference {
            role: PlayerRole::Support,
            rank,
        }]),
        // Legacy compatibility for old rosters stored before multi-role support.
        // We keep FLEX-specific tolerance at the DB boundary only.
        "FLEX" => Ok(vec![
            RolePreference {
                role: PlayerRole::Dps,
                rank,
            },
            RolePreference {
                role: PlayerRole::Tank,
                rank,
            },
            RolePreference {
                role: PlayerRole::Support,
                rank,
            },
        ]),
        other => Err(internal_error(format!("invalid player role in DB: {other}"))),
    }
}

fn push_role_preferences(target: &mut Vec<RolePreference>, preferences: Vec<RolePreference>) {
    for preference in preferences {
        if target.iter().any(|existing| existing.role == preference.role) {
            continue;
        }
        target.push(preference);
    }
}

fn primary_role_from_db(
    role_str: &str,
    preferences: &[RolePreference],
) -> Result<PlayerRole, crate::shared::errors::ApiError> {
    if role_str == "FLEX" {
        return preferences
            .first()
            .map(|preference| preference.role)
            .ok_or_else(|| internal_error("legacy FLEX player has no expanded role preferences"));
    }

    PlayerRole::try_from(role_str)
        .map_err(|_| internal_error(format!("invalid player role in DB: {role_str}")))
}

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
    /// None → show ACTIVE + ENDED (default public view).
    /// Some(Draft/Active/Ended) → filter to that specific status.
    pub status_filter: Option<EventStatus>,
    /// When true, the None status_filter default is bypassed and all statuses
    /// are shown (used for owner-scoped listings so drafts are visible).
    pub include_drafts: bool,
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
            (SELECT COUNT(*) FROM events WHERE deleted_at IS NULL) AS total_events,
            (
                SELECT COUNT(*)
                FROM event_players ep
                JOIN events e ON e.id = ep.event_id
                WHERE e.deleted_at IS NULL
            ) AS total_signups,
            (
                SELECT COUNT(*)
                FROM events e
                WHERE e.start_date IS NOT NULL
                  AND e.deleted_at IS NULL
                  AND e.start_date >= NOW()
                  AND e.start_date <= NOW() + INTERVAL '7 days'
            ) AS upcoming_events_this_week,
            (
                SELECT COUNT(*)
                FROM events e
                WHERE e.event_type = 'TOURNEY'
                  AND e.deleted_at IS NULL
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
                     AND deleted_at IS NULL
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
           AND deleted_at IS NULL
           AND start_date >= NOW()
           AND status = 'ACTIVE'
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
                 WHERE deleted_at IS NULL
                     AND status = 'ACTIVE'
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
    query_builder.push(" AND e.deleted_at IS NULL");

    match options.status_filter {
        Some(EventStatus::Draft)  => { query_builder.push(" AND e.status = 'DRAFT'"); }
        Some(EventStatus::Active) => { query_builder.push(" AND e.status = 'ACTIVE'"); }
        Some(EventStatus::Ended)  => { query_builder.push(" AND e.status = 'ENDED'"); }
        // Default: hide drafts on the public view; show all when the listing is
        // scoped to the owner themselves (so they can see their own drafts).
        None if !options.include_drafts => { query_builder.push(" AND e.status IN ('ACTIVE', 'ENDED')"); }
        None => {}
    };

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
    let row = sqlx::query("SELECT id FROM events WHERE id = $1 AND deleted_at IS NULL")
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
    require_discord: bool,
    require_battletag: bool,
    discord_announce: bool,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO events (id, name, description, start_date, event_type, format, public_signup_enabled, max_players, signup_token, status, require_discord, require_battletag, discord_announce)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'DRAFT', $10, $11, $12)",
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
    .bind(require_discord)
    .bind(require_battletag)
    .bind(discord_announce)
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
    require_discord: bool,
    require_battletag: bool,
    discord_announce: Option<bool>,
) -> Result<bool, crate::shared::errors::ApiError> {
    let updated = sqlx::query(
        "UPDATE events
         SET name = $1, description = $2, start_date = $3, event_type = $4, format = $5, max_players = $6,
             require_discord = $7, require_battletag = $8, discord_announce = COALESCE($9::bool, discord_announce)
            WHERE id = $10 AND deleted_at IS NULL
         RETURNING id",
    )
    .bind(name)
    .bind(description)
    .bind(start_date)
    .bind(event_type)
    .bind(format)
    .bind(max_players)
    .bind(require_discord)
    .bind(require_battletag)
    .bind(discord_announce)
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
    let result = sqlx::query(
        "UPDATE events
         SET deleted_at = NOW(),
             is_featured = FALSE
         WHERE id = $1 AND deleted_at IS NULL",
    )
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
    let row = sqlx::query("SELECT max_players FROM events WHERE id = $1 AND deleted_at IS NULL")
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
    let row = sqlx::query("SELECT event_type FROM events WHERE id = $1 AND deleted_at IS NULL")
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?;

    let Some(row) = row else {
        return Ok(None);
    };

    let event_type_db: String = row.get("event_type");
    let event_type = EventType::try_from(event_type_db.as_str())
           .map_err(|_| internal_error(format!("invalid event type in DB: {event_type_db}")))?;

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
    user_id: Option<Uuid>,
) -> Result<(), crate::shared::errors::ApiError> {
    let player_id = Uuid::new_v4();
    let mut tx = pool.begin().await.map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO event_players (id, event_id, name, role, rank, user_id) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(player_id)
    .bind(event_id)
    .bind(name)
    .bind(role)
    .bind(rank)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO event_player_roles (id, event_player_id, role, rank, display_order) VALUES ($1, $2, $3, $4, 0)",
    )
    .bind(Uuid::new_v4())
    .bind(player_id)
    .bind(role)
    .bind(rank)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;
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

pub async fn update_event_player_by_id_in_tx(
    tx: &mut Transaction<'_, Postgres>,
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
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(updated.is_some())
}

/// Replaces all role preferences for a player.
/// Pass an empty slice to clear all roles.
pub async fn replace_player_roles(
    tx: &mut Transaction<'_, Postgres>,
    player_id: Uuid,
    roles: &[(&str, &str)],
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query("DELETE FROM event_player_roles WHERE event_player_id = $1")
        .bind(player_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;

    for (i, (role, rank)) in roles.iter().enumerate() {
        sqlx::query(
            "INSERT INTO event_player_roles (id, event_player_id, role, rank, display_order)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(player_id)
        .bind(role)
        .bind(rank)
        .bind(i as i32)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    }

    Ok(())
}

pub async fn upsert_event_player_team_membership(
    pool: &PgPool,
    event_id: Uuid,
    team_id: Uuid,
    player_id: Uuid,
    assigned_role: Option<&str>,
    assigned_rank: Option<&str>,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id, assigned_role, assigned_rank)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (event_id, event_player_id)
         DO UPDATE SET event_team_id = EXCLUDED.event_team_id,
                       assigned_role = EXCLUDED.assigned_role,
                       assigned_rank = EXCLUDED.assigned_rank",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(team_id)
    .bind(player_id)
    .bind(assigned_role)
    .bind(assigned_rank)
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
    let result = sqlx::query("INSERT INTO event_teams (id, event_id, name) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(team_name)
        .execute(pool)
        .await;

    match result {
        Ok(_) => Ok(true),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Ok(false),
        Err(e) => Err(internal_error(e)),
    }
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
        Ok(Some(_)) => Ok(TeamNameUpdateOutcome::Updated),
        Ok(None) => Ok(TeamNameUpdateOutcome::NotFound),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Ok(TeamNameUpdateOutcome::DuplicateName),
        Err(e) => Err(internal_error(e)),
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
    assigned_role: Option<&str>,
    assigned_rank: Option<&str>,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "INSERT INTO event_team_members (id, event_id, event_team_id, event_player_id, assigned_role, assigned_rank)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(event_id)
    .bind(team_id)
    .bind(player_id)
    .bind(assigned_role)
    .bind(assigned_rank)
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

pub async fn delete_event_team_by_id_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let deleted =
        sqlx::query("DELETE FROM event_teams WHERE id = $1 AND event_id = $2 RETURNING id")
            .bind(team_id)
            .bind(event_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(internal_error)?;

    Ok(deleted.is_some())
}

pub async fn clear_team_from_event_matches_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
    team_id: Uuid,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE event_matches
         SET team_a_id = NULL, updated_at = NOW()
         WHERE event_id = $1 AND team_a_id = $2",
    )
    .bind(event_id)
    .bind(team_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "UPDATE event_matches
         SET team_b_id = NULL, updated_at = NOW()
         WHERE event_id = $1 AND team_b_id = $2",
    )
    .bind(event_id)
    .bind(team_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn insert_event_player_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_id: Uuid,
    name: &str,
    role: &str,
    rank: &str,
    signup_request_id: Option<Uuid>,
    user_id: Option<Uuid>,
    roles: &[(&str, &str)],
) -> Result<(), crate::shared::errors::ApiError> {
    let player_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO event_players (id, event_id, name, role, rank, signup_request_id, user_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(player_id)
    .bind(event_id)
    .bind(name)
    .bind(role)
    .bind(rank)
    .bind(signup_request_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    for (i, (r, rk)) in roles.iter().enumerate() {
        sqlx::query(
            "INSERT INTO event_player_roles (id, event_player_id, role, rank, display_order) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(player_id)
        .bind(r)
        .bind(rk)
        .bind(i as i32)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    }

    Ok(())
}

pub async fn update_signup_request_status_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
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
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    Ok(result.rows_affected())
}

pub async fn is_event_owner(
    pool: &PgPool,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id
             FROM event_memberships
             WHERE event_id = $1
               AND user_id = $2
               AND role = 'owner'
               AND EXISTS (
                   SELECT 1
                   FROM events e
                   WHERE e.id = event_memberships.event_id
                     AND e.deleted_at IS NULL
               )",
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
    let row = sqlx::query("SELECT signup_token FROM events WHERE id = $1 AND deleted_at IS NULL")
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
    let result = sqlx::query("UPDATE events SET signup_token = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(signup_token)
        .bind(event_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(result.rows_affected() > 0)
}

pub async fn is_user_already_a_player(
    pool: &PgPool,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id
             FROM event_players
             WHERE event_id = $1
               AND user_id = $2
             LIMIT 1",
    )
    .bind(event_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn user_has_identity_for_provider(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id FROM auth_identities
         WHERE user_id = $1 AND provider = $2
         LIMIT 1",
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn event_signup_info_by_token(
    pool: &PgPool,
    signup_token: &str,
    viewer_user_id: Option<Uuid>,
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
                e.status,
                e.public_signup_enabled,
                e.require_discord,
                e.require_battletag,
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
                         WHERE e.signup_token = $1
                             AND e.deleted_at IS NULL",
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
           .map_err(|_| internal_error(format!("invalid event type in DB: {event_type_db}")))?;
    let format_db: String = row.get("format");
    let format = EventFormat::try_from(format_db.as_str())
           .map_err(|_| internal_error(format!("invalid event format in DB: {format_db}")))?;

    let current_players_i64: i64 = row.get("current_players");
    let current_players = usize::try_from(current_players_i64)
        .map_err(|_| internal_error(format!("invalid current_players in DB: {current_players_i64}")))?;

    let current_signup_requests_i64: i64 = row.get("current_signup_requests");
    let current_signup_requests = usize::try_from(current_signup_requests_i64)
        .map_err(|_| internal_error(format!("invalid current_signup_requests in DB: {current_signup_requests_i64}")))?;

    let max_players = u8::try_from(row.get::<i32, _>("max_players"))
        .map_err(|_| internal_error("invalid max_players in DB"))?;

    let status_db: String = row.get("status");
    let status = EventStatus::try_from(status_db.as_str())
        .map_err(|_| internal_error(format!("invalid event status in DB: {status_db}")))?;
    let public_signup_enabled: bool = row.get("public_signup_enabled");
    let require_discord: bool = row.get("require_discord");
    let require_battletag: bool = row.get("require_battletag");
    let event_id: Uuid = row.get("id");

    let already_joined = if let Some(uid) = viewer_user_id {
        is_user_already_a_player(pool, event_id, uid).await?
    } else {
        false
    };

    Ok(Some(PublicEventSignupInfo {
        event_id,
        event_name: row.get("name"),
        event_description: row.get("description"),
        start_date: row.get::<Option<OffsetDateTime>, _>("start_date"),
        event_type,
        format,
        max_players,
        current_players,
        current_signup_requests,
        status,
        public_signup_enabled,
        require_discord,
        require_battletag,
        already_joined,
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
         WHERE id = $3 AND deleted_at IS NULL
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
        sqlx::query("UPDATE events SET is_featured = FALSE WHERE is_featured = TRUE AND id <> $1 AND deleted_at IS NULL")
            .bind(event_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;

        sqlx::query("UPDATE events SET is_featured = TRUE WHERE id = $1 AND deleted_at IS NULL")
            .bind(event_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    } else {
        sqlx::query("UPDATE events SET is_featured = FALSE WHERE id = $1 AND deleted_at IS NULL")
            .bind(event_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    Ok(())
}

pub async fn set_event_status(
    pool: &PgPool,
    event_id: Uuid,
    status: EventStatus,
) -> Result<(), crate::shared::errors::ApiError> {
    sqlx::query(
        "UPDATE events SET status = $1 WHERE id = $2 AND deleted_at IS NULL"
    )
        .bind(status.as_db_value())
        .bind(event_id)
        .execute(pool)
        .await
        .map_err(internal_error)?;

    Ok(())
}

pub async fn create_signup_request(
    pool: &PgPool,
    event_id: Uuid,
    name: &str,
    user_id: Option<Uuid>,
    roles: &[RolePreferenceInput],
    reported_discord: Option<&str>,
    reported_battletag: Option<&str>,
) -> Result<(), crate::shared::errors::ApiError> {
    let request_id = Uuid::new_v4();
    let mut tx = pool.begin().await.map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO event_signup_requests (id, event_id, name, status, user_id, reported_discord, reported_battletag)
             VALUES ($1, $2, $3, 'pending', $4, $5, $6)",
    )
    .bind(request_id)
    .bind(event_id)
    .bind(name)
    .bind(user_id)
    .bind(reported_discord)
    .bind(reported_battletag)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    for (i, rp) in roles.iter().enumerate() {
        sqlx::query(
            "INSERT INTO event_signup_request_roles
                 (id, signup_request_id, role, rank, display_order)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(request_id)
        .bind(rp.role.trim())
        .bind(rp.rank.trim())
        .bind(i as i32)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;
    Ok(())
}

pub async fn has_pending_signup_request_with_user_id(
    pool: &PgPool,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT id
             FROM event_signup_requests
             WHERE event_id = $1
               AND status = 'pending'
               AND user_id = $2
             LIMIT 1",
    )
    .bind(event_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn has_pending_signup_request_with_battletag(
    pool: &PgPool,
    event_id: Uuid,
    battletag: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "-- pending request with matching reported_battletag
         SELECT id FROM event_signup_requests
         WHERE event_id = $1
           AND status = 'pending'
           AND LOWER(reported_battletag) = LOWER($2)
         UNION ALL
         -- pending request whose linked user has this battletag
         SELECT esr.id FROM event_signup_requests esr
         JOIN auth_identities ai ON ai.user_id = esr.user_id
           AND ai.provider = 'battlenet'
           AND LOWER(ai.provider_username) = LOWER($2)
         WHERE esr.event_id = $1
           AND esr.status = 'pending'
         UNION ALL
         -- accepted player whose linked user has this battletag
         SELECT ep.id FROM event_players ep
         JOIN auth_identities ai ON ai.user_id = ep.user_id
           AND ai.provider = 'battlenet'
           AND LOWER(ai.provider_username) = LOWER($2)
         WHERE ep.event_id = $1
         LIMIT 1",
    )
    .bind(event_id)
    .bind(battletag)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    Ok(row.is_some())
}

pub async fn has_pending_signup_request_with_discord(
    pool: &PgPool,
    event_id: Uuid,
    discord: &str,
) -> Result<bool, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "-- pending request with matching reported_discord
         SELECT id FROM event_signup_requests
         WHERE event_id = $1
           AND status = 'pending'
           AND LOWER(reported_discord) = LOWER($2)
         UNION ALL
         -- pending request whose linked user has this discord
         SELECT esr.id FROM event_signup_requests esr
         JOIN auth_identities ai ON ai.user_id = esr.user_id
           AND ai.provider = 'discord'
           AND LOWER(ai.provider_username) = LOWER($2)
         WHERE esr.event_id = $1
           AND esr.status = 'pending'
         UNION ALL
         -- accepted player whose linked user has this discord
         SELECT ep.id FROM event_players ep
         JOIN auth_identities ai ON ai.user_id = ep.user_id
           AND ai.provider = 'discord'
           AND LOWER(ai.provider_username) = LOWER($2)
         WHERE ep.event_id = $1
         LIMIT 1",
    )
    .bind(event_id)
    .bind(discord)
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
        "SELECT esr.id, esr.event_id, esr.name, esr.status, esr.user_id,
                esr.reported_discord, esr.reported_battletag,
                u.username, u.display_name,
                (SELECT ai.provider_username FROM auth_identities ai
                 WHERE ai.user_id = esr.user_id AND ai.provider = 'discord' LIMIT 1) AS discord_username,
                (SELECT ai.provider_username FROM auth_identities ai
                 WHERE ai.user_id = esr.user_id AND ai.provider = 'battlenet' LIMIT 1) AS battletag
             FROM event_signup_requests esr
             LEFT JOIN users u ON u.id = esr.user_id
             WHERE esr.event_id = $1
             ORDER BY esr.created_at DESC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    if rows.is_empty() {
        return Ok(vec![]);
    }

    let request_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();
    let role_rows = sqlx::query(
        "SELECT signup_request_id, role, rank
         FROM event_signup_request_roles
         WHERE signup_request_id = ANY($1)
         ORDER BY signup_request_id, display_order ASC",
    )
    .bind(&request_ids[..])
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let mut roles_by_request: std::collections::HashMap<Uuid, Vec<RolePreference>> =
        std::collections::HashMap::new();
    for rr in role_rows {
        let req_id: Uuid = rr.get("signup_request_id");
        let role_str: String = rr.get("role");
        let rank_str: String = rr.get("rank");
        let rank = parse_player_rank_db(rank_str.as_str())?;
        let preferences = expand_db_role_preferences(role_str.as_str(), rank)?;
        let target = roles_by_request.entry(req_id).or_default();
        push_role_preferences(target, preferences);
    }

    let mut requests = Vec::with_capacity(rows.len());
    for row in rows {
        let id: Uuid = row.get("id");
        let status_str: String = row.get("status");
        let status = SignupStatus::try_from(status_str.as_str())
            .map_err(|_| internal_error(format!("invalid signup status in DB: {status_str}")))?;
        let roles = roles_by_request.remove(&id).unwrap_or_default();
        let submitter_user_id: Option<Uuid> = row.get("user_id");
        let linked_user = submitter_user_id.map(|uid| LinkedUserInfo {
            id: uid,
            username: row.get::<Option<String>, _>("username").unwrap_or_default(),
            display_name: row.get::<Option<String>, _>("display_name").unwrap_or_default(),
            discord_username: row.get("discord_username"),
            battletag: row.get("battletag"),
            avatar_url: None,
        });
        requests.push(EventSignupRequest {
            id,
            event_id: row.get("event_id"),
            name: row.get("name"),
            roles,
            status,
            linked_user,
            reported_discord: row.get("reported_discord"),
            reported_battletag: row.get("reported_battletag"),
            submitter_user_id,
        });
    }

    Ok(requests)
}

pub async fn get_signup_request(
    pool: &PgPool,
    event_id: Uuid,
    request_id: Uuid,
) -> Result<Option<EventSignupRequest>, crate::shared::errors::ApiError> {
    let row = sqlx::query(
        "SELECT esr.id, esr.event_id, esr.name, esr.status, esr.user_id,
                esr.reported_discord, esr.reported_battletag,
                u.username, u.display_name,
                (SELECT ai.provider_username FROM auth_identities ai
                 WHERE ai.user_id = esr.user_id AND ai.provider = 'discord' LIMIT 1) AS discord_username,
                (SELECT ai.provider_username FROM auth_identities ai
                 WHERE ai.user_id = esr.user_id AND ai.provider = 'battlenet' LIMIT 1) AS battletag
             FROM event_signup_requests esr
             LEFT JOIN users u ON u.id = esr.user_id
             WHERE esr.event_id = $1 AND esr.id = $2",
    )
    .bind(event_id)
    .bind(request_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Ok(None);
    };

    let role_rows = sqlx::query(
        "SELECT role, rank
         FROM event_signup_request_roles
         WHERE signup_request_id = $1
         ORDER BY display_order ASC",
    )
    .bind(request_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let roles: Vec<RolePreference> = {
        let mut acc = Vec::with_capacity(role_rows.len());
        for rr in role_rows {
            let role_str: String = rr.get("role");
            let rank_str: String = rr.get("rank");
            let rank = parse_player_rank_db(rank_str.as_str())?;
            let preferences = expand_db_role_preferences(role_str.as_str(), rank)?;
            push_role_preferences(&mut acc, preferences);
        }
        acc
    };

    let status_str: String = row.get("status");
    let status = SignupStatus::try_from(status_str.as_str())
        .map_err(|_| internal_error(format!("invalid signup status in DB: {status_str}")))?;
    let submitter_user_id: Option<Uuid> = row.get("user_id");
    let linked_user = submitter_user_id.map(|uid| LinkedUserInfo {
        id: uid,
        username: row.get::<Option<String>, _>("username").unwrap_or_default(),
        display_name: row.get::<Option<String>, _>("display_name").unwrap_or_default(),
        discord_username: row.get("discord_username"),
        battletag: row.get("battletag"),
        avatar_url: None,
    });
    Ok(Some(EventSignupRequest {
        id: row.get("id"),
        event_id: row.get("event_id"),
        name: row.get("name"),
        roles,
        status,
        linked_user,
        reported_discord: row.get("reported_discord"),
        reported_battletag: row.get("reported_battletag"),
        submitter_user_id,
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
            e.status,
            e.signup_token,
            e.public_signup_enabled,
            e.require_discord,
            e.require_battletag,
            e.discord_announce,
            e.max_players,
            m.user_id AS creator_id,
            u.display_name AS creator_name
         FROM events e
         LEFT JOIN event_memberships m ON m.event_id = e.id AND m.role = 'owner'
         LEFT JOIN users u ON u.id = m.user_id
                 WHERE e.id = $1
                     AND e.deleted_at IS NULL",
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
           .map_err(|_| internal_error(format!("invalid event type in DB: {db_event_type}")))?;
    let db_format: String = row.get("format");
    let format = EventFormat::try_from(db_format.as_str())
           .map_err(|_| internal_error(format!("invalid event format in DB: {db_format}")))?;
    let players = load_event_players_for_event(pool, db_id).await?;
    let teams = load_event_teams_for_event(pool, db_id).await?;
    let matches = load_matches_for_event(pool, db_id, &players).await?;

    Ok(Event {
        id: db_id,
        name: row.get("name"),
        description: row.get("description"),
        start_date: row.get::<Option<OffsetDateTime>, _>("start_date"),
        event_type,
        format,
        is_featured: row.get("is_featured"),
        status: {
            let s: String = row.get("status");
            EventStatus::try_from(s.as_str())
                .map_err(|_| internal_error(format!("invalid event status in DB: {s}")))?       
        },
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
        require_discord: row.get("require_discord"),
        require_battletag: row.get("require_battletag"),
        discord_announce: row.get("discord_announce"),
        max_players: i32_to_u8(row.get::<i32, _>("max_players"), "max_players")?,
        players,
        teams,
        matches,
    })
}

pub async fn load_matches_for_event(
    pool: &PgPool,
    event_id: Uuid,
    players: &[Player],
) -> Result<Vec<Match>, crate::shared::errors::ApiError> {
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
            status: {
                let s: String = row.get("status");
                MatchStatus::try_from(s.as_str())
                    .map_err(|_| internal_error(format!("invalid match status in DB: {s}")))?
            },
            created_at: row.get::<OffsetDateTime, _>("created_at"),
            updated_at: row.get::<OffsetDateTime, _>("updated_at"),
            start_date: row.get::<Option<OffsetDateTime>, _>("start_date"),
            players: players.to_vec(),
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
            ep.user_id AS linked_user_id,
            et.id AS team_id,
            et.name AS team_name,
            etm.assigned_role,
            etm.assigned_rank,
            u.username AS linked_username,
            u.display_name AS linked_display_name,
            (SELECT ai.provider_username
             FROM auth_identities ai
             WHERE ai.user_id = ep.user_id AND ai.provider = 'discord'
             LIMIT 1) AS linked_discord_username,
            (SELECT ai.provider_username
             FROM auth_identities ai
             WHERE ai.user_id = ep.user_id AND ai.provider = 'battlenet'
             LIMIT 1) AS linked_battletag,
            u.avatar_url AS linked_avatar_url,
            esr.reported_discord,
            esr.reported_battletag
         FROM event_players ep
         LEFT JOIN event_team_members etm ON etm.event_player_id = ep.id
         LEFT JOIN event_teams et ON et.id = etm.event_team_id
         LEFT JOIN users u ON u.id = ep.user_id
         LEFT JOIN event_signup_requests esr ON esr.id = ep.signup_request_id
         WHERE ep.event_id = $1
         ORDER BY ep.id ASC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let player_ids: Vec<Uuid> = rows.iter().map(|r| r.get::<Uuid, _>("id")).collect();
    let mut roles_by_player: std::collections::HashMap<Uuid, Vec<RolePreference>> =
        std::collections::HashMap::new();

    if !player_ids.is_empty() {
        let role_rows = sqlx::query(
            "SELECT event_player_id, role, rank
             FROM event_player_roles
             WHERE event_player_id = ANY($1)
             ORDER BY event_player_id, display_order ASC",
        )
        .bind(&player_ids[..])
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;

        for rr in role_rows {
            let pid: Uuid = rr.get("event_player_id");
            let role_str: String = rr.get("role");
            let rank_str: String = rr.get("rank");
            let rank = parse_player_rank_db(rank_str.as_str())?;
            let preferences = expand_db_role_preferences(role_str.as_str(), rank)?;
            let target = roles_by_player.entry(pid).or_default();
            push_role_preferences(target, preferences);
        }
    }

    let mut players = Vec::with_capacity(rows.len());
    for row in rows {
        let role_str: String = row.get("role");
        let rank_str: String = row.get("rank");
        let rank = parse_player_rank_db(rank_str.as_str())?;
        let player_id: Uuid = row.get("id");
        let mut roles = roles_by_player.remove(&player_id).unwrap_or_default();
        if roles.is_empty() {
            push_role_preferences(&mut roles, expand_db_role_preferences(role_str.as_str(), rank)?);
        }
        let assigned_role = row
            .get::<Option<String>, _>("assigned_role")
            .as_deref()
                .map(|s| {
                    PlayerRole::try_from(s)
                        .map_err(|_| internal_error(format!("invalid assigned player role in DB: {s}")))
                })
                .transpose()?;
        let assigned_rank = row
            .get::<Option<String>, _>("assigned_rank")
            .as_deref()
                .map(|s| {
                    PlayerRank::try_from(s)
                        .map_err(|_| internal_error(format!("invalid assigned player rank in DB: {s}")))
                })
                .transpose()?;
        let linked_user_id: Option<Uuid> = row.get("linked_user_id");
        let linked_user = linked_user_id.map(|id| {
            LinkedUserInfo {
                id,
                username: row.get::<Option<String>, _>("linked_username").unwrap_or_default(),
                display_name: row.get::<Option<String>, _>("linked_display_name").unwrap_or_default(),
                discord_username: row.get("linked_discord_username"),
                battletag: row.get("linked_battletag"),
                avatar_url: row.get("linked_avatar_url"),
            }
        });
        players.push(Player {
            id: player_id,
            name: row.get("name"),
            role: primary_role_from_db(role_str.as_str(), &roles)?,
            rank,
            team_id: row.get::<Option<Uuid>, _>("team_id"),
            team: row.get("team_name"),
            assigned_role,
            assigned_rank,
            roles,
            linked_user,
            reported_discord: row.get("reported_discord"),
            reported_battletag: row.get("reported_battletag"),
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

    // Load all team memberships for this event in a single query.
    let member_rows = sqlx::query(
        "SELECT event_team_id, event_player_id
         FROM event_team_members
         WHERE event_id = $1
         ORDER BY event_player_id ASC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    // Group player IDs by team.
    let mut members_by_team: std::collections::HashMap<Uuid, Vec<Uuid>> = std::collections::HashMap::new();
    for member in member_rows {
        let team_id: Uuid = member.get("event_team_id");
        let player_id: Uuid = member.get("event_player_id");
        members_by_team.entry(team_id).or_default().push(player_id);
    }

    Ok(team_rows
        .into_iter()
        .map(|row| {
            let team_id: Uuid = row.get("id");
            let player_ids = members_by_team.remove(&team_id).unwrap_or_default();
            EventTeam {
                id: team_id,
                name: row.get("name"),
                player_ids,
            }
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Participated-events query (for profile page)
// ---------------------------------------------------------------------------

pub struct ParticipatedEventRow {
    pub id: Uuid,
    pub name: String,
    pub start_date: Option<OffsetDateTime>,
    pub event_type: EventType,
    pub format: EventFormat,
    pub status: EventStatus,
}

pub async fn list_participated_events(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<ParticipatedEventRow>, crate::shared::errors::ApiError> {
    let rows = sqlx::query(
        "SELECT e.id, e.name, e.start_date, e.event_type, e.format, e.status
         FROM events e
         INNER JOIN event_players ep ON ep.event_id = e.id
         WHERE ep.user_id = $1
           AND e.deleted_at IS NULL
           AND e.status != 'DRAFT'
         GROUP BY e.id
         ORDER BY e.start_date IS NULL, e.start_date DESC, e.id DESC
         LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let event_type_str: String = row.get("event_type");
        let event_type = EventType::try_from(event_type_str.as_str())
            .map_err(|_| internal_error(format!("invalid event type in DB: {event_type_str}")))?;
        let format_str: String = row.get("format");
        let format = EventFormat::try_from(format_str.as_str())
            .map_err(|_| internal_error(format!("invalid event format in DB: {format_str}")))?;
        result.push(ParticipatedEventRow {
            id: row.get("id"),
            name: row.get("name"),
            start_date: row.get("start_date"),
            event_type,
            format,
            status: {
                let s: String = row.get("status");
                EventStatus::try_from(s.as_str())
                    .map_err(|_| internal_error(format!("invalid event status in DB: {s}")))?
            },
        });
    }

    Ok(result)
}
