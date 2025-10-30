use domain::{Task, TaskId, task::{TaskCyclingExt, PureTaskCycling}};

/// Pure function to get task cycle position
pub fn get_task_cycle_position(
    tasks: &[Task],
    task_id: &TaskId,
) -> (usize, usize) {
    let cycling = PureTaskCycling;
    cycling.get_task_position(tasks, task_id)
}