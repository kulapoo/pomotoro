use super::*;
use domain::TaskId;
use usecases::timer::{switch_timer_task, SwitchTimerTaskCmd};

#[tauri::command(rename_all = "snake_case")]
pub async fn switch_active_task(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    let task_id_parsed =
        TaskId::from_string(&task_id).map_err(|_| format!("Invalid task ID: {}", task_id))?;

    let cmd = SwitchTimerTaskCmd {
        task_id: task_id_parsed,
    };

    switch_timer_task(
        timer_repo.inner().clone(),
        task_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .context("infra::commands::timer_cmd::switch_timer_task - Failed to switch timer task")
    .map_err(|e| e.to_string())?;

    timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())
}