pub mod timer {
    // User Commands
    pub const START: &str = "start_timer";
    pub const PAUSE: &str = "pause_timer";
    pub const RESUME: &str = "resume_timer";
    pub const RESET: &str = "reset_timer";
    pub const SKIP_PHASE: &str = "skip_phase";
    pub const GET_STATE: &str = "get_timer_state";
    pub const SWITCH_ACTIVE_TASK: &str = "switch_active_task";

    // Business Events
    pub const PHASE_COMPLETED: &str = "phase_completed";
    pub const SESSION_COMPLETED: &str = "session_completed";
    pub const TIMER_STARTED: &str = "timer_started";
    pub const TIMER_PAUSED: &str = "timer_paused";
    pub const TIMER_RESET: &str = "timer_reset";
    pub const UPDATE_TIMER_SECS: &str = "update_timer_secs";
}

pub mod task {
    // User Commands
    pub const CREATE: &str = "create_task";
    pub const UPDATE: &str = "update_task";
    pub const DELETE: &str = "delete_task";
    pub const RESET: &str = "reset_task";
    pub const GET: &str = "get_task";
    pub const GET_ALL: &str = "get_all_tasks";
    pub const GET_ACTIVE: &str = "get_active_tasks";
    pub const GET_BY_TAGS: &str = "get_tasks_by_tags";
    pub const COMPLETE_TASK: &str = "complete_task";
    pub const RESET_TASK: &str = "reset_task";
    pub const SEARCH: &str = "search_tasks";
    pub const SEARCH_FUZZY: &str = "search_tasks_fuzzy";
    pub const FILTER_BY_STATUS: &str = "filter_tasks_by_status";
    pub const CYCLE_INCOMPLETE_TASK: &str = "cycle_incomplete_task";
    pub const GET_TASK_CYCLE_POSITION: &str = "get_task_cycle_position";
    pub const GET_INCOMPLETE_TASKS: &str = "get_incomplete_tasks";
    pub const DEBUG_CREATE_TEST_TASK: &str = "debug_create_test_task";

    // Business Events
    pub const TASK_CREATED: &str = "task_created";
    pub const TASK_UPDATED: &str = "task_updated";
    pub const TASK_DELETED: &str = "task_deleted";
    pub const TASK_COMPLETED: &str = "task_completed";
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
    pub const GET_EFFECTIVE_AUDIO: &str = "get_effective_audio_config";

    // Business Events
    pub const CONFIG_UPDATED: &str = "config_updated";
    pub const CONFIG_RESET: &str = "config_reset";
}

pub mod audio {
    // User Commands
    pub const TEST_PREVIEW: &str = "test_audio_preview";
    pub const PLAY_NOTIFICATION: &str = "play_notification_sound";
    pub const PLAY_BACKGROUND: &str = "play_background_audio";
    pub const STOP_BACKGROUND: &str = "stop_background_audio";
    pub const GET_LIBRARY: &str = "get_audio_library";
    pub const PLAY: &str = "play_audio";
    pub const STOP: &str = "stop_audio";
    pub const PAUSE: &str = "pause_audio";
    pub const RESUME: &str = "resume_audio";
    pub const SET_VOLUME: &str = "set_audio_volume";
    pub const GET_ACTIVE_PLAYBACKS: &str = "get_active_playbacks";
    pub const STOP_ALL: &str = "stop_all_audio";
    pub const ADD_CUSTOM_ASSET: &str = "add_custom_audio_asset";
    pub const REMOVE_ASSET: &str = "remove_audio_asset";
    pub const CLEANUP_FINISHED: &str = "cleanup_finished_audio";
}

pub mod storage {
    // User Commands
    pub const OPEN_DATA_DIR: &str = "open_data_directory";
    pub const CLEAR_ALL_DATA: &str = "clear_all_data";
    pub const VALIDATE_PATH: &str = "validate_storage_path";
    pub const UPDATE_PATH: &str = "update_storage_path";
    pub const EXPORT_SETTINGS: &str = "export_settings";
    pub const IMPORT_SETTINGS: &str = "import_settings";
}

pub mod notification {
    // User Commands
    pub const TEST: &str = "test_notification";
    pub const REQUEST_PERMISSION: &str = "request_notification_permission";
}

pub mod task_settings {
    // User Commands
    pub const UPDATE: &str = "update_task_settings";
    pub const RESET_TO_DEFAULTS: &str = "reset_task_settings_to_defaults";
    pub const GET_EFFECTIVE: &str = "get_task_effective_settings";
}
