-- Users whose events get announced to a Discord guild.
-- The guild owner is automatically added on guild creation.
CREATE TABLE discord_guild_members (
    discord_guild_id UUID        NOT NULL REFERENCES discord_guilds(id) ON DELETE CASCADE,
    user_id          UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (discord_guild_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_discord_guild_members_user ON discord_guild_members(user_id);

-- Seed: auto-enroll existing guild owners.
INSERT INTO discord_guild_members (discord_guild_id, user_id)
SELECT id, owner_user_id
FROM discord_guilds
WHERE owner_user_id IS NOT NULL
ON CONFLICT DO NOTHING;

-- Per-guild event post tracking (replaces the global discord_message_id sentinel).
CREATE TABLE discord_guild_posts (
    discord_guild_id UUID        NOT NULL REFERENCES discord_guilds(id) ON DELETE CASCADE,
    event_id         UUID        NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    posted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (discord_guild_id, event_id)
);
CREATE INDEX IF NOT EXISTS idx_discord_guild_posts_event ON discord_guild_posts(event_id);

-- Seed: treat all previously-posted events as posted for every existing guild
-- so the bot does not re-announce them.
INSERT INTO discord_guild_posts (discord_guild_id, event_id)
SELECT dg.id, e.id
FROM discord_guilds dg
CROSS JOIN events e
WHERE e.discord_message_id IS NOT NULL
ON CONFLICT DO NOTHING;

-- discord_message_id is superseded by discord_guild_posts; drop it.
ALTER TABLE events DROP COLUMN IF EXISTS discord_message_id;
