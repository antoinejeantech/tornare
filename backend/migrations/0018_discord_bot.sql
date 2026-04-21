-- Track the Discord message posted when an event is published.
-- Used to avoid duplicate posts and to edit/delete the message later.
ALTER TABLE events ADD COLUMN IF NOT EXISTS discord_message_id TEXT;
