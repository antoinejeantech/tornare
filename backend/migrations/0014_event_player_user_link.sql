-- Link event_players and event_signup_requests rows to registered users.
-- Both columns are nullable so that anonymous participants continue to work.
ALTER TABLE event_players
    ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE event_signup_requests
    ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE event_signup_requests
    ADD COLUMN IF NOT EXISTS reported_discord   VARCHAR(100),
    ADD COLUMN IF NOT EXISTS reported_battletag VARCHAR(100);

CREATE INDEX IF NOT EXISTS idx_event_players_user_id
    ON event_players(user_id)
    WHERE user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_event_signup_requests_user_id
    ON event_signup_requests(user_id)
    WHERE user_id IS NOT NULL;
