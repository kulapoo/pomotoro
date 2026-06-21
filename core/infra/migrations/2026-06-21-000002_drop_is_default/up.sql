-- Remove the "default task" special-case.
--
-- All tasks are now equal: any task can be edited or deleted, the
-- starter task created on first boot is just a regular task, and the
-- timer no longer needs a "default" to fall back on (it can run with
-- no task attached).
--
-- See core/usecases/src/bootstrap.rs for the first-boot starter task
-- creation that replaced this.

-- Drop the lookup index first (must precede column drop on some
-- SQLite versions; harmless in any order on >= 3.35).
DROP INDEX IF EXISTS idx_tasks_default;

-- SQLite >= 3.35 supports DROP COLUMN directly. libsqlite3-sys 0.35
-- bundles a newer version, so this is safe.
ALTER TABLE tasks DROP COLUMN is_default;
