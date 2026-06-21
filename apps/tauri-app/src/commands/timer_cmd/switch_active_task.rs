use super::*;
use domain::TaskId;
use usecases::task::{
    SwitchActiveTaskCmd, switch_active_task as switch_active_task_usecase,
};

use infra::adapters::TimerTickService;

#[tauri::command(rename_all = "snake_case")]
pub async fn switch_active_task(
    task_id: String,
    old_task_id: Option<String>,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    timer_tick_service_arc: State<'_, Arc<TimerTickService>>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    let task_id_parsed = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    let old_task_id_parsed = old_task_id
        .map(|id| TaskId::from_string(&id))
        .transpose()
        .map_err(|_| "Invalid old task ID".to_string())?;

    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::switch_active_task - Failed to stop tick loop: {}",
                e
            )
        })?;

    let cmd = SwitchActiveTaskCmd {
        task_id: task_id_parsed,
        old_task_id: old_task_id_parsed,
    };

    switch_active_task_usecase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .context("infra::commands::timer_cmd::switch_active_task - Failed to switch task")
    .map_err(|e| e.to_string())?;

    timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}
