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
    let _orchestration_lock =
        timer_tick_service_arc.inner().orchestration_lock().await;
    let timer_repo_arc = timer_repo.inner().clone();
    let task_repo_arc = task_repo.inner().clone();

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
        task_repo_arc.clone(),
        timer_repo_arc.clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .context("infra::commands::timer_cmd::switch_active_task - Failed to switch task")
    .map_err(|e| e.to_string())?;

    // The tick service keeps its own in-memory Timer, which still references the
    // previous task. Without resyncing it, any later tick loop would emit TICK
    // payloads carrying the old task's remaining seconds and pin the tray
    // countdown to the stale value.
    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::timer_cmd::switch_active_task - Failed to resync tick service timer: {}",
                e
            )
        })?;

    // If the new task's timer is running, restart the tick loop with the new
    // task's config so the live countdown (and tray) reflects the new task
    // instead of freezing after the stop above.
    let new_timer = timer_repo_arc
        .get()
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())?;

    if new_timer.is_running() {
        let task = task_repo_arc
            .get_by_id(task_id_parsed)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task {} not found", task_id))?;
        timer_tick_service_arc
            .start_timer_tick_loop(Some(task.config().timer.clone()), None)
            .await
            .map_err(|e| {
                format!(
                    "infra::commands::timer_cmd::switch_active_task - Failed to restart tick loop: {}",
                    e
                )
            })?;
    }

    Ok(new_timer)
}
