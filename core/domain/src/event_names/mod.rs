pub mod commands;
pub mod ui_listeners;

pub mod timer {
    // Re-export specific items to avoid name conflicts
    // From commands
    pub use super::commands::timer::{
        GET_STATE, SESSION_COMPLETED, SKIP_PHASE, START, SWITCH_ACTIVE_TASK,
        TIMER_PAUSED, TIMER_RESET, TIMER_STARTED,
    };
    // From ui_listeners - use different names for conflicting items
    pub use super::ui_listeners::timer::{
        PAUSE as UI_PAUSE, PHASE_COMPLETED, PHASE_EVENT, PHASE_SKIPPED,
        RESET as UI_RESET, STATUS_CHANGED, TICK,
    };
    // Re-export the command versions with prefix for clarity
    pub use super::commands::timer::{PAUSE as CMD_PAUSE, RESET as CMD_RESET};
}

pub mod task {
    // Commands
    pub use super::commands::task::{
        COMPLETE_TASK, CREATE, CYCLE_INCOMPLETE_TASK, DEBUG_CREATE_TEST_TASK,
        DELETE, FILTER_BY_STATUS, GET, GET_ACTIVE, GET_ALL, GET_BY_TAGS,
        GET_INCOMPLETE_TASKS, GET_TASK_CYCLE_POSITION, RESET, RESET_TASK,
        SEARCH, SEARCH_FUZZY, TASK_CREATED, TASK_DELETED, TASK_UPDATED, UPDATE,
    };
    // Business event (domain event)
    pub use super::commands::task::TASK_COMPLETED as TASK_COMPLETED_EVENT;
    // UI Listeners
    pub use super::ui_listeners::task::*;
}

pub mod config {
    pub use super::commands::config::*;
    pub use super::ui_listeners::config::{
        CONFIG_UPDATED as CONFIG_UPDATED_UI, THEME_CHANGED,
    };
}

pub mod app {
    pub use super::ui_listeners::app::*;
}

pub mod audio {
    pub use super::commands::audio::*;
}

pub mod storage {
    pub use super::commands::storage::*;
}

pub mod notification {
    pub use super::commands::notification::*;
}

pub mod task_settings {
    pub use super::commands::task_settings::*;
}
