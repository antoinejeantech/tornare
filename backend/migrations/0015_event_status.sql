-- Replace the is_ended boolean with a proper status column.
-- Values: 'DRAFT', 'ACTIVE', 'ENDED'.
--
-- DRAFT:  not visible in public listings, registrations not accepted.
-- ACTIVE: live and visible; registrations controlled separately by public_signup_enabled.
-- ENDED:  visible in public listings (shown by default), no new registrations.

ALTER TABLE events ADD COLUMN status TEXT NOT NULL DEFAULT 'ACTIVE';

UPDATE events
   SET status = CASE WHEN is_ended = TRUE THEN 'ENDED' ELSE 'ACTIVE' END;

ALTER TABLE events DROP COLUMN is_ended;
