use super::*;
use usecases::timer::reset_timer_session;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_timer(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<(Timer, Task), String> {
    let timer_repo_arc = timer_repo.inner().clone();

    // Get current timer to find active task
    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::reset_timer - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    // Get the active task ID from the timer
    let task_id = current_timer
        .active_task_id()
        .ok_or("No active task in timer")?;

    reset_timer_session(
        task_id,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer - Failed to reset timer to initial state")
    .map_err(|e| e.to_string())?;

    let timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())?;

    let task = task_repo
        .get_by_id(task_id)
        .await
        .context("infra::commands::timer_cmd::reset_timer - Failed to get task")
        .map_err(|e| e.to_string())?
        .ok_or("infra::commands::timer_cmd::reset_timer - Task not found")?;

    Ok((timer, task))
}