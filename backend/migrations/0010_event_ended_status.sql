-- Add is_ended flag to events so they can be closed without being deleted.
-- Ended events are hidden from the default event listing but remain accessible
-- by direct link and can be re-opened by the event owner.
ALTER TABLE events ADD COLUMN is_ended BOOLEAN NOT NULL DEFAULT FALSE;
