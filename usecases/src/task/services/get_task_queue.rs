use domain::{Task, TaskId, task::{TaskCycling, PureTaskCycling}};
use super::TaskQueueInfo;

/// Pure function to get task queue information
pub fn get_task_queue(
    all_tasks: &[Task],
    include_completed: bool,
    active_task_id: Option<&TaskId>,
) -> TaskQueueInfo {
    let cycling = PureTaskCycling;

    // Filter tasks based on include_completed flag
    let tasks = if include_completed {
        all_tasks.to_vec()
    } else {
        cycling.filter_active_tasks(all_tasks)
    };

    let total_tasks = tasks.len();
    let active_tasks = tasks.iter().filter(|t| !t.is_completed()).count();
    let completed_tasks = tasks.iter().filter(|t| t.is_completed()).count();

    let current_position = if let Some(active_id) = active_task_id {
        tasks.iter().position(|t| &t.id == active_id)
    } else {
        None
    };

    TaskQueueInfo {
        tasks,
        active_task_id: active_task_id.cloned(),
        current_position,
        total_tasks,
        active_tasks,
        completed_tasks,
    }
}