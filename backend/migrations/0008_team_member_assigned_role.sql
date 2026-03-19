-- Store the role a player was assigned to play within a specific team,
-- set either by the auto-balance algorithm or by a manual owner assignment.
-- Kept separate from event_players.role so the player's stated primary
-- preference is never mutated by team assignment operations.
ALTER TABLE event_team_members ADD COLUMN assigned_role TEXT;
ALTER TABLE event_team_members ADD COLUMN assigned_rank TEXT;
