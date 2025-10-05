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
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTimerTaskCmd,
) -> Result<()> {
    // Verify task exists and is not completed
    let mut task = task_repo
        .get_by_id(cmd.task_id)
        .await?
        .ok_or(Error::TaskNotFound { id: cmd.task_id.to_string() })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Load the timer aggregate
    let mut timer = timer_repo.get().await?;

    // If there's a previous active task, set it back to Queued
    if let Some(prev_task_id) = timer.active_task_id() {
        if prev_task_id != cmd.task_id {
            if let Some(mut prev_task) = task_repo.get_by_id(prev_task_id).await? {
                if prev_task.status != domain::TaskStatus::Completed {
                    prev_task.queue()?;
                    let prev_task_id = prev_task.id;
                    task_repo.update(prev_task).await?;
                    // Emit TaskUpdated event for previous task
                    event_publisher.publish(Box::new(domain::TaskUpdated::new(
                        prev_task_id,
                        None,
                        None,
                        None,
                        None,
                        1,
                    )));
                }
            }
        }
    }

    // Set the new task status to Active
    task.activate()?;
    let task_id = task.id;
    task_repo.update(task).await?;

    // Emit TaskUpdated event for new active task
    event_publisher.publish(Box::new(domain::TaskUpdated::new(
        task_id,
        None,
        None,
        None,
        None,
        1,
    )));

    // Execute domain logic: switch the active task
    timer.set_active_task(cmd.task_id);

    // Save the timer state
    timer_repo.save(&timer).await?;

    Ok(())
}
