//! Task settings command handlers module
//!
//! This module contains all task settings-related command handlers that serve as entry points
//! for external task settings management requests.

// Common imports and types used across task settings commands
pub use crate::adapters::events::mem_event_bus::EventPublisherArc;
pub use domain::{Config, TaskId, TaskRepository};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use tauri::{command, State};

// Common types shared across commands
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskSettingsCmd {
    pub task_id: String,
    pub settings: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSettingsResponse {
    pub task_id: String,
    pub settings: Option<Config>,
}

// Declare submodules
mod reset_task_settings_to_defaults;
mod update_task_settings;

// Re-export all command functions
pub use reset_task_settings_to_defaults::reset_task_settings_to_defaults;
pub use update_task_settings::update_task_settings;