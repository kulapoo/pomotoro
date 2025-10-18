use super::*;
use anyhow::{Context, anyhow};
use domain::TimerRepository;
use log::info;
use usecases::task::complete_task as complete_task_uc;
use usecases::timer::complete_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn complete_task(
    task_id: String,
    timer_repo: State<'_, Arc<dyn TimerRepository + Send + Sync>>,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
) -> Result<TaskDto, String> {
    info!("Completing task: id={}", task_id);

    let task_id_parsed = TaskId::from_string(&task_id)
        .context("Invalid task ID")
        .map_err(|e| e.to_string())?;

    // Complete the task (all sessions)
    complete_task_uc(&task_repo, &event_publisher, task_id_parsed)
        .await
        .with_context(|| format!("Failed to complete task: {}", task_id))
        .map_err(|e| {
            log::error!("Failed to complete task {}: {}", task_id, e);
            e.to_string()
        })?;

    complete_timer_phase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .with_context(|| {
        format!("Failed to complete timer phase for task: {}", task_id)
    })
    .map_err(|e| {
        log::error!(
            "Failed to complete timer phase for task {}: {}",
            task_id,
            e
        );
        e.to_string()
    })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after completing")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after completing"))
        .map_err(|e| e.to_string())?;

    info!("Successfully completed task: id={}", task_id);
    Ok(task.into())
}
