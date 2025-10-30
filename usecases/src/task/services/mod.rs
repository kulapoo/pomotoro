//! Pure application services for task operations
//!
//! This module contains pure, synchronous functions that transform data
//! and perform calculations. These services have no I/O operations and
//! work only with in-memory data.
//!
//! # Architecture
//! - Pure functions only (no async, no I/O)
//! - Use domain services for business logic
//! - Return computed results and DTOs
//! - Fully testable without mocks

mod cycle_incomplete_task;
mod cycle_to_next_task;
mod get_active_task_queue;
mod get_incomplete_task_info;
mod get_next_task;
mod get_next_task_for_switch;
mod get_task_cycle_info;
mod get_task_cycle_position;
mod get_task_queue;
mod get_task_queue_summary;
mod get_task_queue_with_priorities;
mod validate_task_switch;

use domain::{Task, TaskId};

// Re-export all service functions
pub use cycle_incomplete_task::cycle_incomplete_task;
pub use cycle_to_next_task::cycle_to_next_task;
pub use get_active_task_queue::get_active_task_queue;
pub use get_incomplete_task_info::get_incomplete_task_info;
pub use get_next_task::get_next_task;
pub use get_next_task_for_switch::get_next_task_for_switch;
pub use get_task_cycle_info::get_task_cycle_info;
pub use get_task_cycle_position::get_task_cycle_position;
pub use get_task_queue::get_task_queue;
pub use get_task_queue_summary::get_task_queue_summary;
pub use get_task_queue_with_priorities::get_task_queue_with_priorities;
pub use validate_task_switch::validate_task_switch;

// ============================================================================
// Data Transfer Objects (DTOs)
// ============================================================================

/// Direction for cycling through tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleDirection {
    Next,
    Previous,
}

/// Query parameters for cycling incomplete tasks
#[derive(Debug, Clone)]
pub struct CycleIncompleteTaskQuery {
    pub current_task_id: Option<TaskId>,
    pub direction: CycleDirection,
}

/// Result of cycling through incomplete tasks
#[derive(Debug, Clone)]
pub struct IncompleteCycleResult {
    pub task: Option<Task>,
    pub position: usize,
    pub total_incomplete: usize,
    pub has_more_tasks: bool,
}

/// Query parameters for getting next task
#[derive(Debug, Clone)]
pub struct GetNextTaskQuery {
    pub current_task_id: Option<TaskId>,
}

/// Result of task cycling operation
#[derive(Debug, Clone)]
pub struct TaskCycleResult {
    pub next_task: Option<Task>,
    pub has_more_tasks: bool,
    pub cycle_position: usize,
    pub total_tasks: usize,
}

/// Query parameters for task queue
#[derive(Debug, Clone)]
pub struct TaskQueueQuery {
    pub include_completed: bool,
    pub active_task_id: Option<TaskId>,
}

/// Information about the task queue
#[derive(Debug, Clone)]
pub struct TaskQueueInfo {
    pub tasks: Vec<Task>,
    pub active_task_id: Option<TaskId>,
    pub current_position: Option<usize>,
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
}

impl TaskQueueInfo {
    pub fn active_task_id(&self) -> Option<TaskId> {
        self.active_task_id
    }
}

/// Summary statistics for task queue
#[derive(Debug, Clone)]
pub struct TaskQueueSummary {
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub paused_tasks: usize,
    pub total_sessions: u32,
    pub completed_sessions: u32,
    pub progress_percentage: f64,
}
