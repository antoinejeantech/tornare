ALTER TABLE auth_identities
    ADD COLUMN IF NOT EXISTS provider_username TEXT;

-- Backfill battletag for existing Battle.net connected accounts
UPDATE auth_identities ai
SET provider_username = ugp.handle
FROM user_game_profiles ugp
WHERE ai.user_id = ugp.user_id
  AND ai.provider = 'battlenet'
  AND ugp.game_code = 'overwatch'
  AND ugp.provider = 'battlenet'
  AND ugp.handle IS NOT NULL
  AND ai.provider_username IS NULL;
