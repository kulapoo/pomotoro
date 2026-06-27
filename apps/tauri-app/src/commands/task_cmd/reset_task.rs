use super::*;
use anyhow::{Context, anyhow};
use domain::EventPublisher;
use domain::TaskRepository;
use domain::Timer;
use domain::TimerRepository;
use infra::adapters::TimerTickService;
use log::info;
use usecases::task::reset_task as reset_task_uc;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_task(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, Arc<dyn TimerRepository + Send + Sync>>,
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<(Timer, Task), String> {
    info!("Resetting task: id={}", task_id);

    let timer_tick_service_arc = timer_tick_service.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id).map_err(|e| {
        log::error!("Invalid task ID '{}': {}", task_id, e);
        format!("Invalid task ID: {}", task_id)
    })?;

    reset_task_uc(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        task_id_parsed,
    )
    .await
    .map_err(|e| {
        log::error!("Failed to reset task {}: {}", task_id, e);
        e.to_string()
    })?;

    // Per the tick-loop ownership contract, the orchestrator drives the stop
    // directly. The TaskReset event handler is a UI-only emitter and no longer
    // touches the loop. Stop before load (matches reset_timer_phase and the
    // tray's menu_reset_task); the timer ends in Idle, so no start.
    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::task_cmd::reset_task - Failed to stop tick loop: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::task_cmd::reset_task - Failed to load timer state: {}",
                e
            )
        })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after reset")
        .map_err(|e| {
            log::error!(
                "Failed to retrieve task {} after reset: {}",
                task_id,
                e
            );
            e.to_string()
        })?
        .ok_or_else(|| {
            log::error!("Task not found after reset: {}", task_id);
            anyhow!("Task not found after reset")
        })
        .map_err(|e| e.to_string())?;

    info!(
        "Successfully reset task: id={}, new_status={:?}",
        task_id,
        task.status()
    );

    let timer = timer_repo.get().await.map_err(|e| e.to_string())?;

    Ok((timer, task))
}
