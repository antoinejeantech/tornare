CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    display_name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS auth_identities (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    email_from_provider TEXT,
    UNIQUE(provider, provider_user_id)
);

CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    UNIQUE(user_id, role)
);

CREATE TABLE IF NOT EXISTS auth_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    start_date TEXT,
    signup_token TEXT,
    event_type TEXT NOT NULL,
    format TEXT NOT NULL DEFAULT '5v5',
    max_players INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_events_signup_token ON events(signup_token);

CREATE TABLE IF NOT EXISTS event_players (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    rank TEXT NOT NULL DEFAULT 'Unranked'
);

CREATE TABLE IF NOT EXISTS event_teams (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    UNIQUE(event_id, name)
);

CREATE TABLE IF NOT EXISTS event_matches (
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
    next_match_slot TEXT,
    winner_team_id UUID REFERENCES event_teams(id) ON DELETE SET NULL,
    is_bracket BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'OPEN'
);

CREATE TABLE IF NOT EXISTS event_team_members (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    event_team_id UUID NOT NULL REFERENCES event_teams(id) ON DELETE CASCADE,
    event_player_id UUID NOT NULL REFERENCES event_players(id) ON DELETE CASCADE,
    UNIQUE(event_id, event_player_id)
);

CREATE TABLE IF NOT EXISTS event_memberships (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    UNIQUE(event_id, user_id)
);

CREATE TABLE IF NOT EXISTS event_signup_requests (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    rank TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
