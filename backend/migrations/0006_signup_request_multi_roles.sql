-- Normalize signup request role/rank into a child table so applicants can
-- list up to 3 role preferences (e.g. Tank/Master + DPS/Silver).
CREATE TABLE IF NOT EXISTS event_signup_request_roles (
    id                UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    signup_request_id UUID    NOT NULL REFERENCES event_signup_requests(id) ON DELETE CASCADE,
    role              TEXT    NOT NULL,
    rank              TEXT    NOT NULL,
    display_order     INT     NOT NULL DEFAULT 0
);

-- Move existing single-role rows into the new table before dropping columns.
INSERT INTO event_signup_request_roles (id, signup_request_id, role, rank, display_order)
SELECT gen_random_uuid(), id, role, rank, 0
FROM event_signup_requests;

ALTER TABLE event_signup_requests DROP COLUMN role;
ALTER TABLE event_signup_requests DROP COLUMN rank;

-- Allow the event_players row to remember which signup request it came from so
-- organisers can review the applicant's original preferences from the roster.
-- ON DELETE SET NULL so deleting a request doesn't remove the accepted player.
ALTER TABLE event_players
    ADD COLUMN signup_request_id UUID REFERENCES event_signup_requests(id) ON DELETE SET NULL;
