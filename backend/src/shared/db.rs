use sqlx::PgPool;

pub async fn init_schema(pool: &PgPool) -> anyhow::Result<()> {
    create_core_tables(pool).await?;
    create_event_tables(pool).await?;

    Ok(())
}

async fn create_core_tables(pool: &PgPool) -> anyhow::Result<()> {
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

    Ok(())
}

async fn create_event_tables(pool: &PgPool) -> anyhow::Result<()> {
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
            signup_token TEXT,
            event_type TEXT NOT NULL CHECK (event_type IN ('PUG', 'TOURNEY')),
            max_players INTEGER NOT NULL
        )",
    )
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
            max_players INTEGER NOT NULL,
            round INTEGER,
            position INTEGER,
            next_match_id UUID REFERENCES event_matches(id) ON DELETE SET NULL,
            next_match_slot TEXT CHECK (next_match_slot IN ('A', 'B')),
            winner_team_id UUID REFERENCES event_teams(id) ON DELETE SET NULL,
            is_bracket BOOLEAN NOT NULL DEFAULT FALSE,
            status TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'READY', 'COMPLETED'))
        )",
    )
    .execute(pool)
    .await?;

    // Backfill columns for pre-existing databases created before tourney bracket support.
    sqlx::query("ALTER TABLE event_matches ADD COLUMN IF NOT EXISTS round INTEGER")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE event_matches ADD COLUMN IF NOT EXISTS position INTEGER")
        .execute(pool)
        .await?;
    sqlx::query(
        "ALTER TABLE event_matches ADD COLUMN IF NOT EXISTS next_match_id UUID REFERENCES event_matches(id) ON DELETE SET NULL",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE event_matches ADD COLUMN IF NOT EXISTS next_match_slot TEXT CHECK (next_match_slot IN ('A', 'B'))",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE event_matches ADD COLUMN IF NOT EXISTS winner_team_id UUID REFERENCES event_teams(id) ON DELETE SET NULL",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE event_matches ADD COLUMN IF NOT EXISTS is_bracket BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE event_matches ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'READY', 'COMPLETED'))",
    )
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

    Ok(())
}

