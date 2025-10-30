use domain::{Task, TaskId, task::{TaskCyclingExt, PureTaskCycling}};
use super::IncompleteCycleResult;

/// Pure function to get incomplete task information
pub fn get_incomplete_task_info(
    tasks: &[Task],
    current_task_id: Option<&TaskId>,
) -> IncompleteCycleResult {
    let cycling = PureTaskCycling;

    // Get incomplete tasks only
    let incomplete_tasks = cycling.filter_incomplete_tasks(tasks);
    let total_incomplete = incomplete_tasks.len();

    // Find current task in incomplete queue
    let current_task = if let Some(id) = current_task_id {
        incomplete_tasks.iter().find(|t| &t.id == id).cloned()
    } else {
        None
    };

    // Calculate position
    let position = if let Some(ref task) = current_task {
        incomplete_tasks
            .iter()
            .position(|t| t.id == task.id)
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_incomplete > 1;

    IncompleteCycleResult {
        task: current_task,
        position,
        total_incomplete,
        has_more_tasks,
    }
}