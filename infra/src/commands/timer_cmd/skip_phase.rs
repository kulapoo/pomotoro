use super::*;
use usecases::timer::skip_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn skip_phase(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    app_handle: AppHandle,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    // Get current timer state to find active task
    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::skip_phase - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    // Get the active task ID from the timer
    let task_id = current_timer
        .active_task_id()
        .ok_or("No active task in timer")?;

    skip_timer_phase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        task_id,
    )
    .await
    .context("infra::commands::timer_cmd::skip_phase - Failed to skip to next phase")
    .map_err(|e| e.to_string())?;

    // Get the updated timer state with correct remaining seconds
    let updated_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::skip_phase - Failed to get updated timer state")
        .map_err(|e| e.to_string())?;

    // Send tauri event with full timer state (not just phase)
    app_handle
        .emit(ui_listeners::timer::PHASE_SKIPPED, updated_timer.state())
        .map_err(|e| e.to_string())?;

    Ok(updated_timer)
}