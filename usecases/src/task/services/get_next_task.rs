use domain::{Task, TaskId, task::{TaskCycling, PureTaskCycling}};

/// Pure function to get the next task in the cycle
pub fn get_next_task(
    tasks: &[Task],
    current_task_id: Option<&TaskId>,
) -> Option<Task> {
    let cycling = PureTaskCycling;
    cycling.get_next_task(tasks, current_task_id)
}