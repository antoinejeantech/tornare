-- Mark all currently-active events as legacy so the Discord bot does not
-- retroactively announce events that were already live before the bot was
-- deployed.  The bot polls for rows WHERE discord_message_id IS NULL; any
-- non-NULL value causes it to skip the row.  Newly-activated events will
-- still start with NULL and be announced normally.
UPDATE events
SET discord_message_id = 'legacy'
WHERE status = 'ACTIVE'
  AND discord_message_id IS NULL
  AND deleted_at IS NULL;
