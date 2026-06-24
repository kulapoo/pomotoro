-- Persist the task completion timestamp on the `tasks` table.
--
-- Previously `completed_at` was held only in the in-memory `Task` aggregate
-- and reconstructed by the builder on load. That worked when the builder
-- auto-stamped it for any `Status::Completed` task, but the new pomodoro
-- semantics split "sessions exhausted" (status flips) from "fully done"
-- (trailing break taken, `completed_at` stamped). The timestamp can no
-- longer be derived from status alone, so it must be persisted.

ALTER TABLE tasks ADD COLUMN completed_at TEXT;
