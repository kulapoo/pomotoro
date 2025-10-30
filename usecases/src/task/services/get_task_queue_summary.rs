use domain::{Task, TaskStatus, task::{TaskCycling, PureTaskCycling}};
use super::TaskQueueSummary;

/// Pure function to get task queue summary
pub fn get_task_queue_summary(all_tasks: &[Task]) -> TaskQueueSummary {
    let cycling = PureTaskCycling;

    let total_tasks = all_tasks.len();
    let active_tasks = cycling.filter_active_tasks(all_tasks);
    let active_count = active_tasks.len();
    let completed_count = all_tasks.iter().filter(|t| t.is_completed()).count();
    let paused_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Paused)
        .count();

    let total_sessions: u32 = all_tasks.iter().map(|t| t.max_sessions as u32).sum();
    let completed_sessions: u32 = all_tasks.iter().map(|t| t.current_sessions as u32).sum();

    TaskQueueSummary {
        total_tasks,
        active_tasks: active_count,
        completed_tasks: completed_count,
        paused_tasks: paused_count,
        total_sessions,
        completed_sessions,
        progress_percentage: if total_sessions > 0 {
            (completed_sessions as f64 / total_sessions as f64) * 100.0
        } else {
            0.0
        },
    }
}