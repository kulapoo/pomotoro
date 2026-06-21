-- Best-effort reverse migration.
--
-- Re-adding the column cannot restore the previous "default task"
-- designation since that information is gone. The column comes back
-- as FALSE for every row. Down-migrations are not officially
-- supported; this exists mainly for completeness.

ALTER TABLE tasks ADD COLUMN is_default BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_tasks_default ON tasks(is_default);
