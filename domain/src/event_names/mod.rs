pub mod commands;
pub mod ui_listeners;

pub mod timer {
    // Re-export specific items to avoid name conflicts
    // From commands
    pub use super::commands::timer::{
        START, SKIP_PHASE, GET_STATE, SWITCH_ACTIVE_TASK,
        TIMER_STARTED, TIMER_PAUSED, TIMER_RESET,
        SESSION_COMPLETED
    };
    // From ui_listeners - use different names for conflicting items
    pub use super::ui_listeners::timer::{
        TICK, STATUS_CHANGED, PHASE_EVENT, PHASE_COMPLETED,
        PHASE_SKIPPED,
        PAUSE as UI_PAUSE,
        RESET as UI_RESET
    };
    // Re-export the command versions with prefix for clarity
    pub use super::commands::timer::{PAUSE as CMD_PAUSE, RESET as CMD_RESET};
}

pub mod task {
    // Commands
    pub use super::commands::task::{
        CREATE, UPDATE, DELETE, RESET, GET, GET_ALL, GET_ACTIVE,
        GET_BY_TAGS, COMPLETE_TASK, RESET_TASK, SEARCH, SEARCH_FUZZY,
        FILTER_BY_STATUS, CYCLE_INCOMPLETE_TASK, GET_TASK_CYCLE_POSITION,
        GET_INCOMPLETE_TASKS, DEBUG_CREATE_TEST_TASK, TASK_CREATED,
        TASK_UPDATED, TASK_DELETED,
    };
    // Business event (domain event)
    pub use super::commands::task::TASK_COMPLETED as TASK_COMPLETED_EVENT;
    // UI Listeners
    pub use super::ui_listeners::task::*;
}

pub mod config {
    pub use super::commands::config::*;
    pub use super::ui_listeners::config::{CONFIG_UPDATED as CONFIG_UPDATED_UI, THEME_CHANGED};
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
