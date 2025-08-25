pub mod timer {
    // User Commands
    pub const START: &str = "start_timer";
    pub const PAUSE: &str = "pause_timer";
    pub const RESET: &str = "reset_timer";
    pub const SKIP_PHASE: &str = "skip_phase";
    pub const GET_STATE: &str = "get_timer_state";
    pub const SWITCH_ACTIVE_TASK: &str = "switch_active_task";

    // Business Events
    pub const UPDATE_STATE: &str = "timer_state_updated";
    pub const PHASE_COMPLETE: &str = "phase_completed";
    pub const SESSION_COMPLETED: &str = "session_completed";
    pub const TIMER_STARTED: &str = "timer_started";
    pub const TIMER_PAUSED: &str = "timer_paused";
    pub const TIMER_RESET: &str = "timer_reset";
}

pub mod task {
    // User Commands
    pub const CREATE: &str = "create_task";
    pub const UPDATE: &str = "update_task";
    pub const DELETE: &str = "delete_task";
    pub const GET: &str = "get_task";
    pub const GET_ALL: &str = "get_all_tasks";
    pub const COMPLETE_SESSION: &str = "complete_task_session";
    pub const RESET_SESSIONS: &str = "reset_task_sessions";

    // Business Events
    pub const TASK_CREATED: &str = "task_created";
    pub const TASK_UPDATED: &str = "task_updated";
    pub const TASK_DELETED: &str = "task_deleted";
    pub const TASK_COMPLETED: &str = "task_completed";
    pub const SESSION_COMPLETED: &str = "task_session_completed";
}

pub mod config {
    // User Commands
    pub const GET_GLOBAL: &str = "get_global_config";
    pub const SAVE_GLOBAL: &str = "save_global_config";
    pub const UPDATE_GENERAL: &str = "update_general_config";
    pub const UPDATE_NOTIFICATIONS: &str = "update_notification_config";
    pub const UPDATE_APPEARANCE: &str = "update_appearance_config";
    pub const UPDATE_AUDIO: &str = "update_audio_config";
    pub const UPDATE_TIMINGS: &str = "update_timing_config";
    pub const RESET_TO_DEFAULTS: &str = "reset_config_to_defaults";

    // Business Events
    pub const CONFIG_UPDATED: &str = "config_updated";
    pub const CONFIG_RESET: &str = "config_reset";
}
