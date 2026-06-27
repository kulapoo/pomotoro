use super::*;

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
    let _orchestration_lock =
        timer_tick_service.inner().orchestration_lock().await;
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
    // operation). The orchestrator owns the tick-loop side effects per the
    // ownership contract — no sleep, no reliance on the Reset event handler.
    reset_timer_phase_usecase(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo_arc.clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer_phase - Failed to reset timer phase")
    .map_err(|e| e.to_string())?;

    // Stop the existing loop and refresh the in-memory cache so a fresh
    // loop (if any) sees the reset remaining seconds.
    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer_phase - Failed to stop tick loop: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer_phase - Failed to load timer state: {}",
                e
            )
        })?;

    // Get the updated timer state with the reset remaining seconds.
    let updated_timer = timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd::reset_timer_phase - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())?;

    // Restart the tick loop so a running phase keeps counting down from the
    // full duration. For a paused timer the loop should not run; leave it
    // stopped to preserve the paused state.
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
