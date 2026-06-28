use std::sync::Arc;

use domain::{EventPublisher, Result, TaskId, TaskRepository, TasksCompleted};

/// After a task is completed, publish `TasksCompleted` when no incomplete
/// tasks remain. Called from every completion site (manual `complete_task`,
/// natural break expiry in `complete_timer_phase`, and `skip_timer_phase`)
/// so the "all done" signal fires regardless of the completion path.
pub async fn publish_tasks_completed_if_all_done(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    last_completed_id: TaskId,
) -> Result<()> {
    if task_repo.get_incomplete_tasks().await?.is_empty() {
        event_publisher
            .publish(Box::new(TasksCompleted::new(vec![last_completed_id])));
    }
    Ok(())
}
