ALTER TABLE users
ADD COLUMN IF NOT EXISTS username TEXT;

-- Seed from display_name first for rows that don't already have a value.
UPDATE users
SET username = regexp_replace(lower(COALESCE(display_name, '')), '[^a-z0-9_]+', '', 'g')
WHERE username IS NULL;

-- Guarantee non-empty seed before collision resolution.
UPDATE users
SET username = 'user'
WHERE username IS NULL OR username = '';

-- Resolve collisions deterministically by appending a numeric suffix.
DO $$
DECLARE
    rec RECORD;
    base_username TEXT;
    candidate TEXT;
    suffix INTEGER;
BEGIN
    FOR rec IN
        SELECT id, username, display_name
        FROM users
        ORDER BY created_at, id
    LOOP
        base_username := regexp_replace(lower(COALESCE(rec.username, '')), '[^a-z0-9_]+', '', 'g');

        IF base_username = '' THEN
            base_username := regexp_replace(lower(COALESCE(rec.display_name, '')), '[^a-z0-9_]+', '', 'g');
        END IF;

        IF base_username = '' THEN
            base_username := 'user';
        END IF;

        candidate := base_username;
        suffix := 1;

        WHILE EXISTS (
            SELECT 1
            FROM users u
            WHERE u.username = candidate
              AND u.id <> rec.id
        ) LOOP
            candidate := base_username || suffix::TEXT;
            suffix := suffix + 1;
        END LOOP;

        UPDATE users
        SET username = candidate
        WHERE id = rec.id;
    END LOOP;
END;
$$;

ALTER TABLE users
ALTER COLUMN username SET NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'users_username_key'
    ) THEN
        ALTER TABLE users
        ADD CONSTRAINT users_username_key UNIQUE (username);
    END IF;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'users_username_format_check'
    ) THEN
        ALTER TABLE users
        ADD CONSTRAINT users_username_format_check
        CHECK (
            username = lower(username)
            AND username ~ '^[a-z0-9_]+$'
            AND char_length(username) BETWEEN 3 AND 24
        );
    END IF;
END;
$$;
