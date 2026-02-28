use super::*;
use domain::TaskId;
use usecases::timer::{StartTimerPhaseCmd, start_timer_phase};

#[tauri::command(rename_all = "snake_case")]
pub async fn start_timer(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    info!("Starting timer for task {}", task_id_parsed);

    let cmd = StartTimerPhaseCmd {
        task_id: Some(task_id_parsed),
    };

    start_timer_phase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .context("infra::commands::timer_cmd::start_timer - Failed to execute start timer phase")
    .map_err(|e| e.to_string())?;

    timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}
