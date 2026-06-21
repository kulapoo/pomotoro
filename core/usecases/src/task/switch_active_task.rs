use domain::{
    Error, EventPublisher, Result, TaskActiveChanged, TaskId, TaskRepository,
    TaskUpdated, TimerRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchActiveTaskCmd {
    pub task_id: TaskId,
    pub old_task_id: Option<TaskId>,
}

/// Switch the active task without timer validation (for UI/bootstrap scenarios)
/// This variant allows switching even while the timer is running, which is needed
/// for UI operations and initial setup
pub async fn switch_active_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchActiveTaskCmd,
) -> Result<()> {
    // Get the target task
    let mut task =
        task_repo.get_by_id(cmd.task_id).await?.ok_or_else(|| {
            Error::TaskNotFound {
                id: cmd.task_id.to_string(),
            }
        })?;

    // Check if task is completed (but skip timer check)
    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // if let Some(old_task_id) = cmd.old_task_id {

    // }

    // Get the single timer instance
    let timer = timer_repo.get().await?;

    // Set new task status to Active
    task.activate()?;
    let task_name = task.name().to_string();
    let task_id = task.id();
    task_repo.update(task).await?;

    // Publish TaskUpdated event for new active task
    event_publisher.publish(Box::new(TaskUpdated::new(
        task_id, None, None, None, None, 1,
    )));

    // Create a new timer for the new task, preserving the state from the old timer
    // This allows seamless task switching during active sessions
    let new_timer =
        domain::Timer::with_state(cmd.task_id, timer.state().clone());
    timer_repo.save(&new_timer).await?;

    // Publish TaskActiveChanged event
    let switch_event = TaskActiveChanged::new(
        cmd.old_task_id,
        cmd.task_id,
        format!("Switched to task: {}", task_name),
        1,
    );
    event_publisher.publish(Box::new(switch_event));

    Ok(())
}
