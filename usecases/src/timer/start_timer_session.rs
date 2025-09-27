use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StartTimerSessionCmd {
    pub task_id: Option<TaskId>,
}

/// Start a timer session for a specific task
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
pub async fn start_timer_session(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: StartTimerSessionCmd,
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

    task.increment_session()?;

    task_repo.update(task.clone()).await?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    let mut timer = timer_repo.get().await?;

    timer.set_active_task(task_id);

    let events = timer.start(&task.config.timer)?;

    timer_repo.save(&timer).await?;

    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
