ALTER TABLE discord_guilds
    ADD COLUMN mention_roles TEXT[] NOT NULL DEFAULT '{}';
