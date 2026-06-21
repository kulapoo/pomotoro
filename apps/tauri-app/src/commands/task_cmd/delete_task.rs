use super::*;
use anyhow::Context;
use log::info;
use usecases::task::{DeleteTaskCmd, delete_task as delete_task_usecase};

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_task(
    id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, Arc<dyn TimerRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<(), String> {
    info!("Deleting task: id={}", id);

    let task_id = TaskId::from_string(&id)
        .map_err(|_| format!("Invalid task ID: {}", id))?;

    let cmd = DeleteTaskCmd { id: task_id };
    delete_task_usecase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .with_context(|| format!("Failed to delete task with id: {}", id))
    .map_err(|e| e.to_string())?;

    Ok(())
}
