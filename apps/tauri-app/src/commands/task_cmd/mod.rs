//! Task command handlers module
//!
//! This module contains all task-related command handlers that serve as entry points
//! for external task management requests.

// Common imports and types used across multiple task commands
pub use domain::{
    AudioConfig, ConfigRepository, EventPublisher, Task, TaskId, TaskRepository,
};
pub use infra::adapters::events::mem_event_bus::EventPublisherArc;
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use std::time::Duration;
pub use tauri::State;

// Declare submodules
mod complete_task;
mod create_task;
mod delete_task;
mod filter_tasks_by_status;
mod get_active_tasks;
mod get_all_tasks;
mod get_incomplete_tasks;
mod get_task;
mod get_tasks_by_tags;
mod reset_task;
mod search_tasks;
mod search_tasks_fuzzy;
mod update_task;

// Re-export all command functions and their types
pub use complete_task::complete_task;
pub use create_task::{CreateTaskRequest, create_task};
pub use delete_task::delete_task;
pub use filter_tasks_by_status::{FilterTasksRequest, filter_tasks_by_status};
pub use get_active_tasks::get_active_tasks;
pub use get_all_tasks::get_all_tasks;
pub use get_incomplete_tasks::get_incomplete_tasks;
pub use get_task::get_task;
pub use get_tasks_by_tags::get_tasks_by_tags;
pub use reset_task::reset_task;
pub use search_tasks::{SearchTasksRequest, search_tasks};
pub use search_tasks_fuzzy::search_tasks_fuzzy;
pub use update_task::{UpdateTaskRequest, update_task};
