use domain::{Task, TaskId, task::{TaskCycling, PureTaskCycling}};
use super::TaskQueueInfo;

/// Pure function to get task queue with priority sorting
pub fn get_task_queue_with_priorities(
    tasks: &[Task],
    active_task_id: Option<&TaskId>,
) -> TaskQueueInfo {
    let cycling = PureTaskCycling;

    // Get active tasks only
    let mut active_tasks = cycling.filter_active_tasks(tasks);

    // Sort by priority
    active_tasks.sort_by(|a, b| {
        // Active task comes first
        if let Some(active_id) = active_task_id {
            if &a.id == active_id {
                return std::cmp::Ordering::Less;
            }
            if &b.id == active_id {
                return std::cmp::Ordering::Greater;
            }
        }

        // Then by completion status
        match (a.is_completed(), b.is_completed()) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => {
                // Finally by creation date (newer first for active, older first for completed)
                if !a.is_completed() && !b.is_completed() {
                    b.created_at.cmp(&a.created_at)
                } else {
                    a.created_at.cmp(&b.created_at)
                }
            }
        }
    });

    let total_tasks = active_tasks.len();
    let active_count = active_tasks.iter().filter(|t| !t.is_completed()).count();
    let completed_count = active_tasks.iter().filter(|t| t.is_completed()).count();

    let current_position = if let Some(active_id) = active_task_id {
        active_tasks.iter().position(|t| &t.id == active_id)
    } else {
        None
    };

    TaskQueueInfo {
        tasks: active_tasks,
        active_task_id: active_task_id.cloned(),
        current_position,
        total_tasks,
        active_tasks: active_count,
        completed_tasks: completed_count,
    }
}