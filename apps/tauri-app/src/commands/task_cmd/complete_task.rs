use super::*;
use anyhow::Context;
use domain::TaskId;
use infra::adapters::{TimerRepositoryArc, TimerTickService};
use log::info;
use tauri::AppHandle;

use super::complete_flow::complete_task_flow;

/// Mark a task as completed (force-complete all sessions).
///
/// Delegates to [`complete_task_flow`] for the shared orchestration (stop +
/// reset the timer, optional auto-advance). That same flow is reused by the
/// system-tray "Complete Task" menu item so both entry points behave
/// identically.
#[tauri::command(rename_all = "snake_case")]
pub async fn complete_task(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
    app_handle: AppHandle,
) -> Result<Task, String> {
    let _orchestration_lock =
        timer_tick_service.inner().orchestration_lock().await;
    info!("Completing task: id={}", task_id);

    let task_id_parsed = TaskId::from_string(&task_id)
        .context("Invalid task ID")
        .map_err(|e| e.to_string())?;

    let task = complete_task_flow(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        config_repo.inner().clone(),
        event_publisher.inner().clone(),
        timer_tick_service.inner().clone(),
        app_handle,
    )
    .await
    .map_err(|e| {
        log::error!("Failed to complete task {}: {}", task_id, e);
        e.to_string()
    })?;

    info!("Successfully completed task: id={}", task_id,);
    Ok(task)
}
