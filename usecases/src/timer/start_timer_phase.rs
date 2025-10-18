use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StartTimerPhaseCmd {
    pub task_id: Option<TaskId>,
}

/// Start a timer phase for a specific task
///
/// Sets the active task on the timer and starts it using
/// the task's configuration.
///
/// ## Business Rules
///
/// - Task must exist and not be completed
/// - Timer must not already be running
///
/// ## Dependencies
///
/// - TaskRepository: For task validation and retrieval
/// - TimerRepository: For timer persistence
/// - EventPublisher: For domain event publishing
pub async fn start_timer_phase(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: StartTimerPhaseCmd,
) -> Result<()> {
    let to_invalid_task_data =
        |msg: String| domain::Error::InvalidTaskParams { message: msg };

    let task_id = cmd.task_id.ok_or_else(|| {
        to_invalid_task_data("Task ID is required".to_string())
    })?;

    let mut task =
        task_repo
            .get_by_id(task_id)
            .await?
            .ok_or(Error::TaskNotFound {
                id: task_id.as_str(),
            })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    let mut timer = timer_repo.get().await?;

    // If there's a previous active task, set it back to Queued
    if let Some(prev_task_id) = timer.active_task_id() {
        if prev_task_id != task_id {
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

    // Get the timer config before moving the task
    let timer_config = task.config.timer.clone();
    let task_id_for_event = task.id;

    task_repo.update(task).await?;

    // Emit TaskUpdated event for new active task
    event_publisher.publish(Box::new(domain::TaskUpdated::new(
        task_id_for_event,
        None,
        None,
        None,
        None,
        1,
    )));

    timer.set_active_task(task_id);

    let events = timer.start(&timer_config)?;

    timer_repo.save(&timer).await?;

    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
