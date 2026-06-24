use super::*;
use anyhow::{Context, anyhow};
use infra::adapters::{TimerRepositoryArc, TimerTickService};
use log::info;
use usecases::task::complete_task as complete_task_uc;

/// Mark a task as completed (force-complete all sessions).
///
/// Also stops and resets the timer for the completed task. This orchestration
/// lives in the command layer (not in the `TaskCompleted` event handler)
/// because task completion during an auto-cycle must NOT reset the timer —
/// only the explicit user-initiated complete should.
#[tauri::command(rename_all = "snake_case")]
pub async fn complete_task(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<Task, String> {
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

    // Stop the tick loop and reset the timer to idle for this task. The
    // `TaskCompleted` event handler no longer performs this side effect
    // (it would race with auto-cycling in the timer use cases).
    timer_tick_service
        .inner()
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!("Failed to stop timer tick loop while completing task: {e}")
        })?;

    usecases::timer::reset_timer_to_idle(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("Failed to reset timer to idle after completing task")
    .map_err(|e| e.to_string())?;

    // Drain the async Reset event handler (stop + load_state).
    tokio::time::sleep(Duration::from_millis(100)).await;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after completing")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after completing"))
        .map_err(|e| e.to_string())?;

    info!(
        "Successfully completed task: id={}, name={}",
        task_id,
        task.name()
    );
    Ok(task)
}
