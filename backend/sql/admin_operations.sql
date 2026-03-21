-- Manual admin SQL operations for Tornare.
--
-- Use explicit transactions for write operations so they can be reviewed
-- before commit when running in psql or a DB client.

-- ---------------------------------------------------------------------------
-- Restore a soft-deleted event
-- ---------------------------------------------------------------------------
-- Reverses the event soft delete by clearing deleted_at.
-- This also reopens the event and ensures it is not re-featured automatically.
--
-- Replace :event_id with the target UUID before running.
-- Example: 550e8400-e29b-41d4-a716-446655440000

BEGIN;

UPDATE events
SET deleted_at = NULL,
    is_ended = FALSE,
    is_featured = FALSE
WHERE id = ':event_id'::uuid
  AND deleted_at IS NOT NULL;

COMMIT;
