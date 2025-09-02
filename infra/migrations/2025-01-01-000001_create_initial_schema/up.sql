-- Create timers table (single timer with active task reference)
CREATE TABLE timers (
    id TEXT PRIMARY KEY NOT NULL,
    active_task_id TEXT, -- Reference to the currently active task
    current_phase TEXT NOT NULL DEFAULT 'work',
    remaining_seconds INTEGER NOT NULL DEFAULT 1500,
    is_running BOOLEAN NOT NULL DEFAULT FALSE,
    session_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Create tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    sessions INTEGER NOT NULL DEFAULT 4,
    current_sessions INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'queued',
    tags TEXT, -- JSON array stored as text
    config TEXT NOT NULL, -- JSON object with timer configuration and other settings
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);


-- Create config table (single row table)
CREATE TABLE config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    config_data TEXT NOT NULL, -- JSON object stored as text
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Create session_history table for tracking completed sessions
CREATE TABLE session_history (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    session_type TEXT NOT NULL, -- 'work' or 'break'
    duration_seconds INTEGER NOT NULL,
    completed_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Create indexes
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_default ON tasks(is_default);
CREATE INDEX idx_timers_active_task ON timers(active_task_id);
CREATE INDEX idx_session_history_task_id ON session_history(task_id);
CREATE INDEX idx_session_history_completed_at ON session_history(completed_at);

-- Insert the single default timer
INSERT INTO timers (
    id,
    active_task_id,
    current_phase,
    remaining_seconds,
    is_running,
    session_count,
    created_at,
    updated_at
) VALUES (
    'default-timer-001',
    NULL,
    'work',
    1500,
    FALSE,
    0,
    datetime('now'),
    datetime('now')
);