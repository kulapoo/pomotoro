use super::*;
use usecases::timer::resume_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn resume_timer(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    // Get current timer state to find active task
    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::resume_timer - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    // Check if the timer can be resumed
    let current_state = current_timer.state();
    if current_state.status() == TimerStatus::Idle {
        debug!("Cannot resume timer - timer is idle");
        return Err("Timer is not running. Start a timer first.".to_string());
    }

    if current_state.status() != TimerStatus::Paused {
        debug!("Timer is not paused, cannot resume");
        // Return current state instead of error
        return timer_repo_arc
            .get()
            .await
            .context("infra::commands::timer_cmd - Failed to get timer state")
            .map_err(|e| e.to_string());
    }

    // Get the task ID from the timer
    let task_id = current_timer.task_id();

    info!("Resuming timer for task {}", task_id);

    resume_timer_phase(
        task_id,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::resume_timer - Failed to resume timer")
    .map_err(|e| e.to_string())?;

    timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())
}