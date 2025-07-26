// Event/Command constants for Tauri integration

pub mod timer {
    pub const START: &str = "start_timer";
    pub const PAUSE: &str = "pause_timer";
    pub const RESET: &str = "reset_timer";
    pub const SKIP_PHASE: &str = "skip_phase";
    pub const UPDATE_STATE: &str = "timer_state_updated";
    pub const PHASE_COMPLETE: &str = "phase_completed";
    pub const GET_STATE_WITH_TASK: &str = "get_timer_state_with_task";
    pub const SWITCH_TASK: &str = "switch_task";
}

pub mod task {
    pub const CREATE: &str = "create_task";
    pub const UPDATE: &str = "update_task";
    pub const DELETE: &str = "delete_task";
    pub const GET: &str = "get_task";
    pub const GET_ALL: &str = "get_all_tasks";
}

pub mod config {
    pub const GET_GLOBAL: &str = "get_global_config";
    pub const SAVE_GLOBAL: &str = "save_global_config";
    pub const UPDATE_GENERAL: &str = "update_general_config";
    pub const UPDATE_NOTIFICATIONS: &str = "update_notification_config";
    pub const UPDATE_APPEARANCE: &str = "update_appearance_config";
    pub const UPDATE_AUDIO: &str = "update_audio_config";
    pub const UPDATE_TIMINGS: &str = "update_timing_config";
    pub const RESET_TO_DEFAULTS: &str = "reset_config_to_defaults";
}