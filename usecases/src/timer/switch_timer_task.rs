use domain::timer::TimerService;
use domain::{Error, EventPublisher, Result, TaskId, TaskRepository};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchTimerTaskCmd {
    pub task_id: String,
}

/// Switch the active task for the timer
///
/// This use case changes the active task context for the timer,
/// potentially adjusting timer configuration based on task-specific
/// settings. It validates the task exists and is not completed.
///
/// ## Business Rules
///
/// - Task must exist and not be completed
/// - Switches task context while preserving timer state where appropriate
/// - May adjust timer configuration based on task settings
///
/// ## Dependencies
///
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For task validation and retrieval
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn switch_timer_task(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTimerTaskCmd,
) -> Result<()> {
    let task_id =
        TaskId::from_string(&cmd.task_id).map_err(|_| Error::TaskNotFound {
            id: cmd.task_id.clone(),
        })?;

    // Verify task exists and is not completed
    let task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or(Error::TaskNotFound { id: cmd.task_id })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Switch to the task with its configuration
    timer_service.switch_task(task_id, Some(&task)).await?;

    Ok(())
}
