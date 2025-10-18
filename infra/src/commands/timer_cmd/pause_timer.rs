use super::*;
use usecases::timer::pause_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn pause_timer(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    // Get current timer state to find active task
    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::pause_timer - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    // Check if the timer can be paused
    let current_state = current_timer.state();
    if current_state.status() == TimerStatus::Idle {
        debug!("Cannot pause timer - timer is idle");
        return Err("Timer is not running. Start a timer first.".to_string());
    }

    if current_state.status() == TimerStatus::Paused {
        debug!("Timer is already paused");
        // Return current state instead of error
        return timer_repo_arc
            .get()
            .await
            .context("infra::commands::timer_cmd - Failed to get timer state")
            .map_err(|e| e.to_string());
    }

    // Get the active task ID from the timer
    let task_id = current_timer
        .active_task_id()
        .ok_or("No active task in timer")?;

    info!("Pausing timer for task {}", task_id);

    pause_timer_phase(
        task_id,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::pause_timer - Failed to toggle pause state")
    .map_err(|e| e.to_string())?;

    timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())
}