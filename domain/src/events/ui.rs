
pub mod timer {
    pub const TICK: &str = "timer:tick";
    pub const STATUS_CHANGED: &str = "timer:status_changed";
    pub const PHASE_EVENT: &str = "timer:phase_event";
    pub const STATE_UPDATED: &str = "timer:state_updated";
}

pub mod task {
    pub const LIST_UPDATED: &str = "task:list_updated";
    pub const ACTIVE_CHANGED: &str = "task:active_changed";
    pub const PROGRESS_UPDATED: &str = "task:progress_updated";
}

pub mod config {
    pub const SETTINGS_UPDATED: &str = "config:settings_updated";
    pub const THEME_CHANGED: &str = "config:theme_changed";
}