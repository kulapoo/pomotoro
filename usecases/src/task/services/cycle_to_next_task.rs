use domain::{Task, TaskId, task::{TaskCycling, PureTaskCycling}};
use super::TaskCycleResult;

/// Pure function to cycle to the next task and get cycle information
pub fn cycle_to_next_task(
    tasks: &[Task],
    current_task_id: Option<&TaskId>,
) -> TaskCycleResult {
    let cycling = PureTaskCycling;

    // Get active tasks only
    let active_tasks = cycling.filter_active_tasks(tasks);
    let total_tasks = active_tasks.len();

    // Get next task
    let next_task = cycling.get_next_task(&active_tasks, current_task_id);

    // Calculate cycle position
    let cycle_position = if let Some(next) = &next_task {
        active_tasks
            .iter()
            .position(|t| t.id == next.id)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_tasks > 1;

    TaskCycleResult {
        next_task,
        has_more_tasks,
        cycle_position,
        total_tasks,
    }
}