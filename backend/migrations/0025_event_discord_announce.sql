-- Per-event Discord announcement opt-out.
-- TRUE by default so all existing active events keep current behaviour.
-- The bot checks this alongside guild.announcements_enabled.
ALTER TABLE events
    ADD COLUMN discord_announce BOOLEAN NOT NULL DEFAULT TRUE;
