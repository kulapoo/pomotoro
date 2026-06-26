use domain::{
    Error, EventPublisher, Result, Task, TaskId, TaskRepository, Timer,
    TimerConfiguration, TimerRepository,
};
use std::sync::Arc;

/// Resets multiple completed tasks back to Queued status
pub async fn reset_tasks(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    task_ids: Vec<TaskId>,
) -> Result<(Timer, Vec<Task>)> {
    let mut reset_tasks = Vec::with_capacity(task_ids.len());

    for &task_id in &task_ids {
        let mut task =
            task_repo.get_by_id(task_id).await?.ok_or_else(|| {
                Error::TaskNotFound {
                    id: task_id.to_string(),
                }
            })?;

        task.reset();

        let task_event = task.clone();

        task_repo.update(task.clone()).await?;

        event_publisher.publish(Box::new(domain::TaskReset::new(
            task_id,
            Some(task_event.name().to_string()),
            task_event.description().map(|s| s.to_string()),
            Some(task_event.max_sessions()),
            Some(task_event.tags().to_vec()),
            1,
        )));

        reset_tasks.push(task);
    }

    let mut timer = timer_repo.get().await?;
    timer.reset(&TimerConfiguration::default())?;
    timer_repo.save(&timer).await?;

    Ok((timer, reset_tasks))
}
