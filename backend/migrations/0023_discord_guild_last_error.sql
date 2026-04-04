-- Store the last posting error per guild so the frontend can show a warning
-- when the bot is misconfigured (e.g. missing channel permissions).
ALTER TABLE discord_guilds
    ADD COLUMN last_post_error    TEXT,
    ADD COLUMN last_post_error_at TIMESTAMPTZ;
