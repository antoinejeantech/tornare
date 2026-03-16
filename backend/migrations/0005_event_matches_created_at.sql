ALTER TABLE event_matches
    ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS start_date TIMESTAMPTZ;

-- Auto-update updated_at on every row modification
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER event_matches_set_updated_at
    BEFORE UPDATE ON event_matches
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
