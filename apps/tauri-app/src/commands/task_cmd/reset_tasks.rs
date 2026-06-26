use super::*;
use domain::EventPublisher;
use domain::TaskRepository;
use domain::Timer;
use domain::TimerRepository;
use log::info;
use usecases::task::reset_tasks as reset_tasks_uc;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_tasks(
    task_ids: Vec<String>,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, Arc<dyn TimerRepository + Send + Sync>>,
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
) -> Result<(Timer, Vec<Task>), String> {
    info!("Resetting {} tasks", task_ids.len());

    let parsed_ids = task_ids
        .iter()
        .map(|id| {
            TaskId::from_string(id).map_err(|e| {
                log::error!("Invalid task ID '{}': {}", id, e);
                format!("Invalid task ID: {}", id)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    reset_tasks_uc(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        parsed_ids,
    )
    .await
    .map_err(|e| {
        log::error!("Failed to reset tasks: {}", e);
        e.to_string()
    })
}
