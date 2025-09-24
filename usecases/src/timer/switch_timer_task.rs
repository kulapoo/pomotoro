use domain::{Error, EventPublisher, Result, TaskId, TaskRepository, TimerRepository};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchTimerTaskCmd {
    pub task_id: TaskId,
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
/// - TimerRepository: For timer persistence
/// - TaskRepository: For task validation and retrieval
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn switch_timer_task(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    _event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTimerTaskCmd,
) -> Result<()> {
    // Verify task exists and is not completed
    let task = task_repo
        .get_by_id(cmd.task_id)
        .await?
        .ok_or(Error::TaskNotFound { id: cmd.task_id.to_string() })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Load the timer aggregate
    let mut timer = timer_repo.get().await?;
    
    // Execute domain logic: switch the active task
    timer.set_active_task(cmd.task_id);
    
    // Save the timer state
    timer_repo.save(&timer).await?;

    Ok(())
}
