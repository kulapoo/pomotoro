-- Drop indexes
DROP INDEX IF EXISTS idx_timers_active_task;
DROP INDEX IF EXISTS idx_tasks_default;
DROP INDEX IF EXISTS idx_tasks_status;

-- Drop tables
DROP TABLE IF EXISTS config;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS timers;