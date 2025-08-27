DROP INDEX IF EXISTS idx_session_history_completed_at;
DROP INDEX IF EXISTS idx_session_history_task_id;
DROP INDEX IF EXISTS idx_tasks_default;
DROP INDEX IF EXISTS idx_tasks_status;

DROP TABLE IF EXISTS session_history;
DROP TABLE IF EXISTS config;
DROP TABLE IF EXISTS timer_state;
DROP TABLE IF EXISTS tasks;