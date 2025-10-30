use domain::{Task, TaskId, task::{TaskCycling, PureTaskCycling}};
use super::TaskCycleResult;

/// Pure function to get task cycle information without advancing
pub fn get_task_cycle_info(
    tasks: &[Task],
    current_task_id: Option<&TaskId>,
) -> TaskCycleResult {
    let cycling = PureTaskCycling;

    // Get active tasks only
    let active_tasks = cycling.filter_active_tasks(tasks);
    let total_tasks = active_tasks.len();

    // Find current task
    let current_task = if let Some(current_id) = current_task_id {
        active_tasks.iter().find(|t| &t.id == current_id).cloned()
    } else {
        None
    };

    // Calculate cycle position
    let cycle_position = if let Some(current) = &current_task {
        active_tasks
            .iter()
            .position(|t| t.id == current.id)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_tasks > 1;

    TaskCycleResult {
        next_task: current_task,
        has_more_tasks,
        cycle_position,
        total_tasks,
    }
}