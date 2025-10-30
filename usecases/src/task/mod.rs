//! Task-related use cases and application services
//!
//! This module contains:
//! - Pure application services in `services/` (synchronous, no I/O)
//! - Use cases at the root level (async orchestration with I/O)
//!
//! # Architecture
//! - **Services**: Pure functions for data transformation and calculations
//! - **Use Cases**: Async functions that orchestrate repositories and domain logic

// ============================================================================
// Pure Application Services
// ============================================================================

pub mod services;

// Re-export service functions and DTOs for convenient access
pub use services::{
    // DTOs
    CycleDirection,
    CycleIncompleteTaskQuery,
    GetNextTaskQuery,
    IncompleteCycleResult,
    TaskCycleResult,
    TaskQueueInfo,
    TaskQueueQuery,
    TaskQueueSummary,
    // Service functions
    cycle_incomplete_task,
    cycle_to_next_task,
    get_active_task_queue,
    get_incomplete_task_info,
    get_next_task,
    get_next_task_for_switch,
    get_task_cycle_info,
    get_task_cycle_position,
    get_task_queue,
    get_task_queue_summary,
    get_task_queue_with_priorities,
    validate_task_switch,
};

// Backward compatibility: Keep old submodule structure
// These re-export from services for existing code
pub mod cycle_incomplete_task {
    pub use super::services::{
        CycleDirection, CycleIncompleteTaskQuery, IncompleteCycleResult,
        cycle_incomplete_task, get_incomplete_task_info, get_task_cycle_position,
    };
}

pub mod cycle_task {
    pub use super::services::{
        GetNextTaskQuery, TaskCycleResult, cycle_to_next_task, get_next_task,
        get_task_cycle_info,
    };
}

pub mod get_task_queue {
    pub use super::services::{
        TaskQueueInfo, TaskQueueQuery, TaskQueueSummary, get_active_task_queue,
        get_task_queue, get_task_queue_summary, get_task_queue_with_priorities,
    };
}

// ============================================================================
// Use Cases (Async Orchestration)
// ============================================================================

pub mod complete_task;
pub mod create_task;
pub mod delete_task;
pub mod get_task;
pub mod reset_task;
pub mod reset_task_settings;
pub mod search_tasks;
pub mod set_default_task;
pub mod switch_task;
pub mod update_task;
pub mod update_task_settings;

// Import switch_to_next_task from root level
mod switch_to_next_task;

// Re-export use case functions
pub use complete_task::complete_task;
pub use create_task::{CreateTaskCmd, create_task};
pub use delete_task::{DeleteTaskCmd, delete_task};
pub use get_task::{
    GetTaskQuery, GetTasksQuery, get_task_by_id, get_task_by_tags, get_tasks,
    get_tasks_by_status,
};
pub use reset_task::reset_task;
pub use reset_task_settings::reset_task_settings_to_defaults;
pub use search_tasks::{
    FilterTasksByStatusQuery, SearchTasksQuery, filter_tasks_by_status,
    search_tasks, search_tasks_fuzzy,
};
pub use set_default_task::{
    SetDefaultTaskCmd, get_default_task, set_default_task,
};
pub use switch_task::{SwitchTaskCmd, switch_task};
pub use switch_to_next_task::switch_to_next_task;
pub use update_task::{UpdateTaskCmd, update_task};
pub use update_task_settings::update_task_settings;
