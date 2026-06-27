use super::*;

use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::reset_timer_to_idle;

/// Fully stop the timer and return it to the Idle state, regardless of the
/// current phase. Unlike `reset_timer_phase` (which only restarts the current
/// phase's countdown), this stops the timer entirely.
#[tauri::command(rename_all = "snake_case")]
pub async fn reset_timer(
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

    // Reset the timer to idle (business operation). Publishes a Reset event
    // that is now a UI-only notification (the handler no longer touches the
    // tick loop). Per the tick-loop ownership contract, this orchestrator owns
    // the stop and state refresh.
    reset_timer_to_idle(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer - Failed to reset timer to idle state")
    .map_err(|e| e.to_string())?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer - Failed to load timer state: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::reset_timer - Failed to stop tick loop: {}",
                e
            )
        })?;

    let timer = timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd::reset_timer - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())?;

    info!("Timer reset to idle for task {}", task_id);
    Ok(timer)
}
