use domain::{Task, task::{TaskCycling, PureTaskCycling}};

/// Pure function to get active tasks
pub fn get_active_task_queue(tasks: &[Task]) -> Vec<Task> {
    let cycling = PureTaskCycling;
    cycling.filter_active_tasks(tasks)
}