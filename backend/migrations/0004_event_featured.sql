ALTER TABLE events
    ADD COLUMN IF NOT EXISTS is_featured BOOLEAN;

UPDATE events
SET is_featured = FALSE
WHERE is_featured IS NULL;

ALTER TABLE events
    ALTER COLUMN is_featured SET DEFAULT FALSE;

ALTER TABLE events
    ALTER COLUMN is_featured SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_events_single_featured
    ON events ((is_featured))
    WHERE is_featured = TRUE;