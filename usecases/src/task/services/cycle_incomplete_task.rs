use domain::{Task, TaskId, task::{TaskCycling, TaskCyclingExt, PureTaskCycling}};
use super::{CycleDirection, IncompleteCycleResult};

/// Pure function to cycle through incomplete tasks
pub fn cycle_incomplete_task(
    tasks: &[Task],
    current_task_id: Option<&TaskId>,
    direction: CycleDirection,
) -> IncompleteCycleResult {
    let cycling = PureTaskCycling;

    // Get incomplete tasks only
    let incomplete_tasks = cycling.filter_incomplete_tasks(tasks);
    let total_incomplete = incomplete_tasks.len();

    // Get next or previous task based on direction
    let task = match direction {
        CycleDirection::Next => {
            cycling.get_next_task(&incomplete_tasks, current_task_id)
        }
        CycleDirection::Previous => {
            cycling.get_previous_task(&incomplete_tasks, current_task_id)
        }
    };

    // Calculate position
    let position = if let Some(ref current_task) = task {
        incomplete_tasks
            .iter()
            .position(|t| t.id == current_task.id)
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_incomplete > 1;

    IncompleteCycleResult {
        task,
        position,
        total_incomplete,
        has_more_tasks,
    }
}