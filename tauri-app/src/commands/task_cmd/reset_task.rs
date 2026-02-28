use super::*;
use anyhow::{anyhow, Context};
use domain::Timer;
use log::info;
use usecases::task::reset_task as reset_task_uc;
use domain::TimerRepository;
use domain::EventPublisher;
use domain::TaskRepository;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_task(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, Arc<dyn TimerRepository + Send + Sync>>,
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
) -> Result<(Timer, Task), String> {
    info!(
        "Resetting task: id={}",
        task_id
    );

    let task_id_parsed = TaskId::from_string(&task_id).map_err(|e| {
        log::error!("Invalid task ID '{}': {}", task_id, e);
        format!("Invalid task ID: {}", task_id)
    })?;

    reset_task_uc(task_repo.inner().clone(), timer_repo.inner().clone(), event_publisher.inner().clone(), task_id_parsed)
        .await
        .with_context(|| format!("Failed to reset task: {}", task_id))
        .map_err(|e| {
            log::error!("Failed to reset task {}: {}", task_id, e);
            e.to_string()
        })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after reset")
        .map_err(|e| {
            log::error!("Failed to retrieve task {} after reset: {}", task_id, e);
            e.to_string()
        })?
        .ok_or_else(|| {
            log::error!("Task not found after reset: {}", task_id);
            anyhow!("Task not found after reset")
        })
        .map_err(|e| e.to_string())?;

    info!(
        "Successfully reset task: id={}, new_status={:?}",
        task_id, task.status
    );

    let timer = timer_repo.get().await.map_err(|e| e.to_string())?;

    Ok((timer, task))
}