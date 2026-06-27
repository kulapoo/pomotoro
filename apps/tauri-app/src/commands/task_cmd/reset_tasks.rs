use super::*;
use domain::EventPublisher;
use domain::TaskRepository;
use domain::Timer;
use domain::TimerRepository;
use infra::adapters::TimerTickService;
use log::info;
use usecases::task::reset_tasks as reset_tasks_uc;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_tasks(
    task_ids: Vec<String>,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, Arc<dyn TimerRepository + Send + Sync>>,
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
    timer_tick_service: State<'_, Arc<TimerTickService>>,
) -> Result<(Timer, Vec<Task>), String> {
    info!("Resetting {} tasks", task_ids.len());

    let timer_tick_service_arc = timer_tick_service.inner().clone();

    let parsed_ids = task_ids
        .iter()
        .map(|id| {
            TaskId::from_string(id).map_err(|e| {
                log::error!("Invalid task ID '{}': {}", id, e);
                format!("Invalid task ID: {}", id)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (timer, tasks) = reset_tasks_uc(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        parsed_ids,
    )
    .await
    .map_err(|e| {
        log::error!("Failed to reset tasks: {}", e);
        e.to_string()
    })?;

    // Per the tick-loop ownership contract, the orchestrator drives the stop
    // directly. The TaskReset event handler is a UI-only emitter and no longer
    // touches the loop. The shared timer is a singleton, so a single stop+load
    // after the batch suffices (matches the tray's menu_reset_task); the timer
    // ends in Idle, so no start.
    timer_tick_service_arc
        .stop_timer_tick_loop()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::task_cmd::reset_tasks - Failed to stop tick loop: {}",
                e
            )
        })?;

    timer_tick_service_arc
        .load_state()
        .await
        .map_err(|e| {
            format!(
                "infra::commands::task_cmd::reset_tasks - Failed to load timer state: {}",
                e
            )
        })?;

    Ok((timer, tasks))
}
