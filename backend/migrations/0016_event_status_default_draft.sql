-- Change the default value for the status column to 'DRAFT'
-- so any direct SQL inserts that omit status also land on DRAFT.
ALTER TABLE events ALTER COLUMN status SET DEFAULT 'DRAFT';
