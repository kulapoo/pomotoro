//! Task command handlers module
//!
//! This module contains all task-related command handlers that serve as entry points
//! for external task management requests.

// Common imports and types used across multiple task commands
pub use crate::adapters::events::mem_event_bus::EventPublisherArc;
pub use domain::{AudioConfig, ConfigRepository, EventPublisher, Task, TaskId, TaskRepository};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use std::time::Duration;
pub use tauri::State;

// Declare submodules
mod complete_task;
mod create_task;
mod cycle_incomplete_task;
mod debug_create_test_task;
mod delete_task;
mod filter_tasks_by_status;
mod get_active_tasks;
mod get_all_tasks;
mod get_incomplete_tasks;
mod get_task;
mod get_task_cycle_position;
mod get_tasks_by_tags;
mod reset_task;
mod search_tasks;
mod search_tasks_fuzzy;
mod update_task;

// Re-export all command functions and their types
pub use complete_task::complete_task;
pub use create_task::{create_task, CreateTaskRequest};
pub use cycle_incomplete_task::{
    cycle_incomplete_task, CycleIncompleteTaskRequest, CycleIncompleteTaskResponse,
};
pub use debug_create_test_task::debug_create_test_task;
pub use delete_task::delete_task;
pub use filter_tasks_by_status::{filter_tasks_by_status, FilterTasksRequest};
pub use get_active_tasks::get_active_tasks;
pub use get_all_tasks::get_all_tasks;
pub use get_incomplete_tasks::get_incomplete_tasks;
pub use get_task::get_task;
pub use get_task_cycle_position::get_task_cycle_position;
pub use get_tasks_by_tags::get_tasks_by_tags;
pub use reset_task::reset_task;
pub use search_tasks::{search_tasks, SearchTasksRequest};
pub use search_tasks_fuzzy::search_tasks_fuzzy;
pub use update_task::{update_task, UpdateTaskRequest};