-- Normalize legacy FLEX values left over from earlier schema versions.
-- The API now stores explicit role preferences only, so old FLEX rows must be
-- expanded into concrete Tank/DPS/Support entries.

WITH flex_players AS (
    SELECT id, rank
    FROM event_players
    WHERE role = 'FLEX'
)
UPDATE event_players ep
SET role = 'DPS'
FROM flex_players fp
WHERE ep.id = fp.id;

WITH flex_players AS (
    SELECT id, rank
    FROM event_players
    WHERE role = 'DPS'
      AND id IN (
          SELECT event_player_id
          FROM event_player_roles
          WHERE role = 'FLEX'
      )
), deleted AS (
    DELETE FROM event_player_roles epr
    USING flex_players fp
    WHERE epr.event_player_id = fp.id
      AND epr.role = 'FLEX'
)
INSERT INTO event_player_roles (id, event_player_id, role, rank, display_order)
SELECT gen_random_uuid(), fp.id, expanded.role, fp.rank, expanded.display_order
FROM flex_players fp
CROSS JOIN (
    VALUES
        ('DPS', 0),
        ('Tank', 1),
        ('Support', 2)
) AS expanded(role, display_order)
WHERE NOT EXISTS (
    SELECT 1
    FROM event_player_roles epr
    WHERE epr.event_player_id = fp.id
      AND epr.role = expanded.role
);

WITH flex_signup_requests AS (
    SELECT signup_request_id, MIN(rank) AS rank
    FROM event_signup_request_roles
    WHERE role = 'FLEX'
    GROUP BY signup_request_id
), deleted AS (
    DELETE FROM event_signup_request_roles esrr
    USING flex_signup_requests fsr
    WHERE esrr.signup_request_id = fsr.signup_request_id
      AND esrr.role = 'FLEX'
)
INSERT INTO event_signup_request_roles (id, signup_request_id, role, rank, display_order)
SELECT gen_random_uuid(), fsr.signup_request_id, expanded.role, fsr.rank, expanded.display_order
FROM flex_signup_requests fsr
CROSS JOIN (
    VALUES
        ('DPS', 0),
        ('Tank', 1),
        ('Support', 2)
) AS expanded(role, display_order)
WHERE NOT EXISTS (
    SELECT 1
    FROM event_signup_request_roles esrr
    WHERE esrr.signup_request_id = fsr.signup_request_id
      AND esrr.role = expanded.role
);