use super::*;
use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::skip_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn skip_phase(
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

    skip_timer_phase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        task_id_parsed,
    )
    .await
    .context(
        "infra::commands::timer_cmd::skip_phase - Failed to skip to next phase",
    )
    .map_err(|e| e.to_string())?;

    // Per the tick-loop ownership contract: stop, refresh, then restart so the
    // new phase counts down. The previous PhaseSkippedHandler did not reliably
    // restart the loop, leaving the timer stuck after a skip.
    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::skip_phase - Failed to stop tick loop: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::skip_phase - Failed to load timer state: {}",
                e
            )
        })?;

    let updated_timer = timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd::skip_phase - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())?;

    // Only start a new loop if the post-skip timer is in a running state.
    // A skip that lands on a paused phase (no auto-start) leaves the loop
    // stopped intentionally.
    if updated_timer.is_running() {
        let task = task_repo
            .inner()
            .get_by_id(task_id_parsed)
            .await
            .context(
                "infra::commands::timer_cmd::skip_phase - Failed to load task",
            )
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        timer_tick_service_arc
            .start_timer_tick_loop(Some(task.config().timer.clone()))
            .await
            .map_err(|e| {
                format!(
                    "infra::commands::timer_cmd::skip_phase - Failed to start tick loop: {}",
                    e
                )
            })?;
    }

    Ok(updated_timer)
}
