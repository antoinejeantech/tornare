ALTER TABLE events
ADD COLUMN deleted_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_events_deleted_at ON events(deleted_at);
