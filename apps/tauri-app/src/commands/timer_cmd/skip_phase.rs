use super::*;
use domain::TaskId;
use usecases::timer::skip_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn skip_phase(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

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

    // Get the updated timer state with correct remaining seconds
    let updated_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::skip_phase - Failed to get updated timer state")
        .map_err(|e| e.to_string())?;

    Ok(updated_timer)
}
