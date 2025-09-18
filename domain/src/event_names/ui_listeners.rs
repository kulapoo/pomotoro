pub mod timer {
    pub const TICK: &str = "timer:tick";
    pub const STATUS_CHANGED: &str = "timer:status_changed";
    pub const PHASE_EVENT: &str = "timer:phase_event";
    pub const PHASE_COMPLETED: &str = "timer:phase_completed";
    pub const PHASE_SKIPPED: &str = "timer:phase_skipped";
    pub const STATE_UPDATED: &str = "timer:state_updated";
    pub const RESET: &str = "timer:timer_reset";
    pub const PAUSE: &str = "timer:timer_paused";
}

pub mod task {
    pub const LIST_UPDATED: &str = "task:list_updated";
    pub const ACTIVE_CHANGED: &str = "task:active_changed";
    pub const PROGRESS_UPDATED: &str = "task:progress_updated";
}

pub mod config {
    pub const CONFIG_UPDATED: &str = "config:config_updated";
    pub const THEME_CHANGED: &str = "config:theme_changed";
}

pub mod app {
    pub const APP_STARTED: &str = "app:started";
    pub const APP_EXITED: &str = "app:exited";
}
