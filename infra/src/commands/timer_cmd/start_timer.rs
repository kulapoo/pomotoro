use super::*;
use usecases::timer::{start_timer_phase, StartTimerPhaseCmd};

#[tauri::command(rename_all = "snake_case")]
pub async fn start_timer(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    let current_state = current_timer.state();

    if current_state.status() == TimerStatus::Paused {
        // Get the active task ID from the timer
        let task_id = current_timer
            .active_task_id()
            .ok_or("No active task in timer")?;

        debug!("Resuming paused timer for task {}", task_id);

        // Resume the paused timer
        usecases::timer::resume_timer_phase(
            task_id,
            task_repo.inner().clone(),
            timer_repo.inner().clone(),
            event_publisher.inner().clone(),
        )
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to resume paused timer")
        .map_err(|e| e.to_string())?;

        info!("Resumed timer for task {}", task_id);
    } else {
        // Try to get an active task, or any incomplete task for starting
        let active_tasks = task_repo
            .get_active_tasks()
            .await
            .map_err(|e| e.to_string())?;

        let task = if let Some(active_task) = active_tasks.first() {
            debug!("Using active task: {}", active_task.id);
            active_task.clone()
        } else {
            // No active tasks, try to get any incomplete task
            let incomplete_tasks = task_repo
                .get_incomplete_tasks()
                .await
                .map_err(|e| e.to_string())?;

            incomplete_tasks
                .first()
                .ok_or("No tasks available. Please create a task first.")?
                .clone()
        };

        let task_id = task.id;
        info!("Starting timer for task {}", task_id);

        let cmd = StartTimerPhaseCmd {
            task_id: Some(task_id),
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
    }

    timer_repo
        .inner()
        .clone()
        .get()
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())
}