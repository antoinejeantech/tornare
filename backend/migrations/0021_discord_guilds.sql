-- Multi-guild Discord bot support.
-- A guild owner connects their Discord server by adding the bot and running
-- /setup in the announcement channel. The bot polls this table to know which
-- channels to post to, and the frontend lets owners manage their settings.

CREATE TABLE IF NOT EXISTS discord_guilds (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id              TEXT        NOT NULL UNIQUE,        -- Discord snowflake (server ID)
    guild_name            TEXT,                               -- cached display name
    owner_user_id         UUID        REFERENCES users(id) ON DELETE SET NULL,  -- nullable: slash-cmd setup happens before user links account
    channel_id            TEXT        NOT NULL,               -- announcement channel snowflake
    announcements_enabled BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_discord_guilds_owner ON discord_guilds(owner_user_id);
