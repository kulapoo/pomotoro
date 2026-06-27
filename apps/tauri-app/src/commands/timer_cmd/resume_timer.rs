use super::*;
use domain::TaskId;
use infra::adapters::TimerTickService;
use usecases::timer::resume_timer_phase;

#[tauri::command(rename_all = "snake_case")]
pub async fn resume_timer(
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

    info!("Resuming timer for task {}", task_id_parsed);

    resume_timer_phase(
        task_id_parsed,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
    )
    .await
    .context(
        "infra::commands::timer_cmd::resume_timer - Failed to resume timer",
    )
    .map_err(|e| e.to_string())?;

    // Per the tick-loop ownership contract, this orchestrator owns the restart.
    // Previously there was no TimerResumedHandler, so the loop was never
    // restarted after a resume — the timer appeared stuck.
    let task = task_repo
        .inner()
        .get_by_id(task_id_parsed)
        .await
        .context(
            "infra::commands::timer_cmd::resume_timer - Failed to load task",
        )
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    timer_tick_service_arc
        .start_timer_tick_loop(Some(task.config().timer.clone()), None)
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::resume_timer - Failed to start tick loop: {}",
                e
            )
        })?;

    timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}
