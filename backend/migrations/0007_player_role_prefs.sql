-- Unified role preferences for every event player, regardless of how they
-- were added (manually by the owner or via an accepted signup request).
-- When a signup request is accepted, rows are copied here from
-- event_signup_request_roles so the request tables stay read-only history.
-- Owners can freely add, remove, or reorder rows here at any time.
CREATE TABLE event_player_roles (
    id              UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    event_player_id UUID    NOT NULL REFERENCES event_players(id) ON DELETE CASCADE,
    role            TEXT    NOT NULL,
    rank            TEXT    NOT NULL,
    display_order   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_event_player_roles_player ON event_player_roles(event_player_id);

-- Migrate existing players: copy their current single role/rank from event_players.
INSERT INTO event_player_roles (id, event_player_id, role, rank, display_order)
SELECT gen_random_uuid(), id, role, rank, 0
FROM event_players;

-- Drop the now-redundant event_player_role_prefs table from the previous
-- migration attempt (replaced by this cleaner design).
DROP TABLE IF EXISTS event_player_role_prefs;
