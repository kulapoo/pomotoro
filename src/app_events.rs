pub mod timer {
    pub const GET_STATE: &str = "get_timer_state";
    pub const START: &str = "start_timer";
    pub const PAUSE: &str = "pause_timer";
    pub const RESET: &str = "reset_timer";
    pub const SKIP_PHASE: &str = "skip_phase";
    pub const GET_STATE_WITH_TASK: &str = "get_timer_state_with_task";
    pub const SWITCH_TASK: &str = "switch_active_task";
    pub const PHASE_COMPLETE: &str = "phase-complete";
}

pub mod task {
    pub const CREATE: &str = "create_task";
    pub const GET: &str = "get_task";
    pub const GET_ALL: &str = "get_all_tasks";
    pub const GET_ACTIVE: &str = "get_active_tasks";
    pub const UPDATE: &str = "update_task";
    pub const DELETE: &str = "delete_task";
    pub const GET_BY_TAGS: &str = "get_tasks_by_tags";
    pub const COMPLETE_SESSION: &str = "complete_task_session";
    pub const RESET_SESSIONS: &str = "reset_task_sessions";
}

pub mod config {
    pub const GET_GLOBAL: &str = "get_global_config";
    pub const SAVE_GLOBAL: &str = "save_global_config";
    pub const RESET_TO_DEFAULTS: &str = "reset_global_config_to_defaults";
    pub const UPDATE_TIMINGS: &str = "update_default_timings";
    pub const UPDATE_CYCLE_LENGTH: &str = "update_default_cycle_length";
    pub const UPDATE_GENERAL: &str = "update_general";
    pub const UPDATE_NOTIFICATIONS: &str = "update_notification_preferences";
    pub const UPDATE_APPEARANCE: &str = "update_appearance";
    pub const UPDATE_AUDIO: &str = "update_audio_config";
    pub const GET_EFFECTIVE_TASK_CONFIG: &str = "get_effective_task_config";
    pub const GET_EFFECTIVE_AUDIO_CONFIG: &str = "get_effective_audio_config";
}

pub mod audio {
    pub const GET_LIBRARY: &str = "get_audio_library";
    pub const PLAY: &str = "play_audio";
    pub const STOP: &str = "stop_audio";
    pub const PAUSE: &str = "pause_audio";
    pub const RESUME: &str = "resume_audio";
    pub const SET_VOLUME: &str = "set_audio_volume";
    pub const GET_ACTIVE: &str = "get_active_playbacks";
    pub const STOP_ALL: &str = "stop_all_audio";
    pub const PLAY_NOTIFICATION: &str = "play_notification_sound";
    pub const PLAY_BACKGROUND: &str = "play_background_audio";
    pub const STOP_BACKGROUND: &str = "stop_background_audio";
    pub const ADD_ASSET: &str = "add_custom_audio_asset";
    pub const REMOVE_ASSET: &str = "remove_audio_asset";
    pub const CLEANUP: &str = "cleanup_finished_audio";
    pub const TEST_PREVIEW: &str = "test_audio_preview";
}
