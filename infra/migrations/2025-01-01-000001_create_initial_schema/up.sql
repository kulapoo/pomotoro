-- Create tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    sessions INTEGER NOT NULL DEFAULT 4,
    current_sessions INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'queued',
    tags TEXT, -- JSON array stored as text
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Create timer_state table (single row table)
CREATE TABLE timer_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    timer_config TEXT NOT NULL, -- JSON object stored as text
    current_phase TEXT NOT NULL,
    remaining_seconds INTEGER NOT NULL,
    is_running BOOLEAN NOT NULL DEFAULT FALSE,
    current_task_id TEXT,
    session_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (current_task_id) REFERENCES tasks(id)
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
CREATE INDEX idx_session_history_task_id ON session_history(task_id);
CREATE INDEX idx_session_history_completed_at ON session_history(completed_at);