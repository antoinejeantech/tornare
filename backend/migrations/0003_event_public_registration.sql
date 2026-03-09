ALTER TABLE events
    ADD COLUMN IF NOT EXISTS public_signup_enabled BOOLEAN;

UPDATE events
SET public_signup_enabled = FALSE
WHERE public_signup_enabled IS NULL;

ALTER TABLE events
    ALTER COLUMN public_signup_enabled SET DEFAULT FALSE;

ALTER TABLE events
    ALTER COLUMN public_signup_enabled SET NOT NULL;
