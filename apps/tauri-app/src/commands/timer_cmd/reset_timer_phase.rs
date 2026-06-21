use super::*;
use std::time::Duration;

use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::reset_timer_phase as reset_timer_phase_usecase;

/// Reset the current phase's countdown to its full duration while preserving
/// the timer's running/paused state. A running phase keeps counting down from
/// the full duration; a paused phase stays paused with the reset duration.
///
/// Unlike `reset_timer` (which fully stops the timer to Idle), this only
/// restarts the current phase.
#[tauri::command(rename_all = "snake_case")]
pub async fn reset_timer_phase(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();
    let timer_tick_service_arc = timer_tick_service.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    // Read the task's timer configuration (needed to restart the tick loop).
    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("infra::commands::timer_cmd::reset_timer_phase - Failed to get task")
        .map_err(|e| e.to_string())?
        .ok_or("infra::commands::timer_cmd::reset_timer_phase - Task not found")?;

    // Reset the current phase's countdown to its full duration (business
    // operation). This publishes a Reset event whose handler stops the tick
    // loop. The event bus is fire-and-forget (handlers run on spawned tasks),
    // so drain that handler before restarting the loop below — otherwise its
    // stop_timer_tick_loop() could abort the loop we are about to start.
    reset_timer_phase_usecase(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo_arc.clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer_phase - Failed to reset timer phase")
    .map_err(|e| e.to_string())?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Get the updated timer state with the reset remaining seconds.
    let updated_timer = timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd::reset_timer_phase - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())?;

    // Restart the tick loop so a running phase keeps counting down from the
    // full duration. For a paused timer the loop would no-op, so skip it to
    // preserve the paused state.
    if updated_timer.is_running() {
        timer_tick_service_arc
            .start_timer_tick_loop(Some(task.config().timer.clone()), None)
            .await
            .map_err(|e| {
                format!(
                    "infra::commands::timer_cmd::reset_timer_phase - Failed to restart tick loop: {}",
                    e
                )
            })?;
    }

    info!("Timer phase reset for task {}", task_id);
    Ok(updated_timer)
}
