use crate::shared::{
    errors::{bad_request, internal_error, not_found, ApiError},
    models::{Event, EventTeam, EventType, Match, Player},
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn init_schema(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT,
            display_name TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth_identities (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            provider TEXT NOT NULL,
            provider_user_id TEXT NOT NULL,
            email_from_provider TEXT,
            UNIQUE(provider, provider_user_id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_roles (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK (role IN ('admin', 'moderator', 'user')),
            UNIQUE(user_id, role)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth_sessions (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            refresh_token_hash TEXT NOT NULL UNIQUE,
            expires_at TIMESTAMPTZ NOT NULL,
            revoked_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            start_date TEXT,
            event_type TEXT NOT NULL CHECK (event_type IN ('PUG', 'TOURNEY')),
            max_players INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    // Remove legacy DB-level max_players check constraints.
    sqlx::query("ALTER TABLE events DROP CONSTRAINT IF EXISTS events_max_players_check")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE events ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE events ADD COLUMN IF NOT EXISTS start_date TEXT")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE events ADD COLUMN IF NOT EXISTS signup_token TEXT")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_events_signup_token
         ON events(signup_token)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS event_players (
            id UUID PRIMARY KEY,
            event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            role TEXT NOT NULL,
            rank TEXT NOT NULL DEFAULT 'Unranked'
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS event_teams (
            id UUID PRIMARY KEY,
            event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            UNIQUE(event_id, name)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS event_matches (
            id UUID PRIMARY KEY,
            event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            team_a_id UUID REFERENCES event_teams(id) ON DELETE SET NULL,
            team_b_id UUID REFERENCES event_teams(id) ON DELETE SET NULL,
            title TEXT NOT NULL,
            map TEXT NOT NULL,
            max_players INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE event_matches DROP CONSTRAINT IF EXISTS event_matches_max_players_check")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS event_team_members (
            id UUID PRIMARY KEY,
            event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            event_team_id UUID NOT NULL REFERENCES event_teams(id) ON DELETE CASCADE,
            event_player_id UUID NOT NULL REFERENCES event_players(id) ON DELETE CASCADE,
            UNIQUE(event_id, event_player_id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS event_memberships (
            id UUID PRIMARY KEY,
            event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK (role IN ('owner', 'manager', 'viewer')),
            UNIQUE(event_id, user_id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS event_signup_requests (
            id UUID PRIMARY KEY,
            event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            role TEXT NOT NULL,
            rank TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('pending', 'accepted', 'declined')),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    let event_token_rows = sqlx::query(
        "SELECT id
         FROM events
         WHERE signup_token IS NULL OR signup_token = ''",
    )
    .fetch_all(pool)
    .await?;

    for row in event_token_rows {
        let event_id: Uuid = row.get("id");
        sqlx::query("UPDATE events SET signup_token = $1 WHERE id = $2")
            .bind(Uuid::new_v4().to_string())
            .bind(event_id)
            .execute(pool)
            .await?;
    }

    // Legacy data backfill: ensure every event has an owner membership so creator metadata resolves.
    let orphan_event_rows = sqlx::query(
        "SELECT e.id
         FROM events e
         LEFT JOIN event_memberships m ON m.event_id = e.id AND m.role = 'owner'
         WHERE m.id IS NULL",
    )
    .fetch_all(pool)
    .await?;

    if !orphan_event_rows.is_empty() {
        let fallback_owner_row = sqlx::query(
            "SELECT id
             FROM users
             WHERE is_active = TRUE
             ORDER BY created_at ASC
             LIMIT 1",
        )
        .fetch_optional(pool)
        .await?;

        if let Some(fallback_owner_row) = fallback_owner_row {
            let fallback_owner_id: Uuid = fallback_owner_row.get("id");

            for orphan_event_row in orphan_event_rows {
                let orphan_event_id: Uuid = orphan_event_row.get("id");

                sqlx::query(
                    "INSERT INTO event_memberships (id, event_id, user_id, role)
                     VALUES ($1, $2, $3, $4)
                     ON CONFLICT (event_id, user_id) DO NOTHING",
                )
                .bind(Uuid::new_v4())
                .bind(orphan_event_id)
                .bind(fallback_owner_id)
                .bind("owner")
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn load_match(pool: &PgPool, match_id: Uuid) -> Result<Match, ApiError> {
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
            g.max_players
         FROM event_matches g
         LEFT JOIN event_teams ta ON ta.id = g.team_a_id
         LEFT JOIN event_teams tb ON tb.id = g.team_b_id
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
        players,
    })
}

pub async fn load_matches_for_event(pool: &PgPool, event_id: Uuid) -> Result<Vec<Match>, ApiError> {
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

pub async fn load_event(pool: &PgPool, event_id: Uuid) -> Result<Event, ApiError> {
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

pub async fn load_event_players_for_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<Vec<Player>, ApiError> {
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
) -> Result<Vec<EventTeam>, ApiError> {
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

pub fn i64_to_usize(value: i64, label: &str) -> Result<usize, ApiError> {
    usize::try_from(value).map_err(|_| bad_request(&format!("Invalid {label} value")))
}

pub fn i32_to_u8(value: i32, label: &str) -> Result<u8, ApiError> {
    u8::try_from(value).map_err(|_| bad_request(&format!("Invalid {label} value")))
}

pub fn i32_to_usize(value: i32, label: &str) -> Result<usize, ApiError> {
    usize::try_from(value).map_err(|_| bad_request(&format!("Invalid {label} value")))
}
