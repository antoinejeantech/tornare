ALTER TABLE event_matches
    ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS start_date TIMESTAMPTZ;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'events'
          AND column_name = 'start_date'
          AND data_type = 'text'
    ) THEN
        EXECUTE $migration$
            ALTER TABLE events
            ALTER COLUMN start_date TYPE TIMESTAMPTZ
            USING (
                CASE
                    WHEN start_date IS NULL OR btrim(start_date) = '' THEN NULL
                    WHEN start_date ~ '(Z|[+-][0-9]{2}:[0-9]{2})$' THEN start_date::timestamptz
                    WHEN start_date ~ '^[0-9]{4}-[0-9]{2}-[0-9]{2}[T ][0-9]{2}:[0-9]{2}(:[0-9]{2}(\.[0-9]{1,6})?)?$' THEN start_date::timestamp AT TIME ZONE 'UTC'
                    ELSE NULL
                END
            )
        $migration$;
    END IF;
END $$;
