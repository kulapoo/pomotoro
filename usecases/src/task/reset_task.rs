use domain::{Error, EventPublisher, Result, Task, TaskId, TaskRepository, Timer, TimerRepository};
use std::sync::Arc;

use crate::timer::reset_timer_phase;


/// Resets a completed task back to Queued status with optional session reset
pub async fn reset_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
) -> Result<(Task, Timer)> {
    let mut task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_string(),
        }
    })?;

    reset_timer_phase(task_id, task_repo.clone(), timer_repo.clone(), event_publisher.clone()).await?;

    task.reset();

    let task_event = task.clone();


    let mut timer = timer_repo.get().await?;

    let timer_config = &task.get_config().timer;
    timer.reset(timer_config)?;

    task_repo.update(task.clone()).await?;
    timer_repo.save(&timer).await?;

    event_publisher.publish(Box::new(domain::TaskReset::new(task_id, Some(task_event.name), task_event.description, Some(task_event.max_sessions), Some(task_event.tags), 1)));


    Ok((task, timer))
}