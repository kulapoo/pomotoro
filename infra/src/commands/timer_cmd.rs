use crate::adapters::events::mem_event_bus::EventPublisherArc;
use crate::adapters::TimerRepositoryArc;
use std::sync::Arc;
use domain::TaskRepository;
use anyhow::Context;
use domain::{TimerState, event_names::ui_listeners};
use tauri::{AppHandle, Emitter, State};
use tracing::info;

use usecases::timer::{
    StartTimerSessionCmd, SwitchTimerTaskCmd,
    get_timer_state as app_get_timer_state, pause_timer_session,
    reset_timer_session, skip_timer_phase, start_timer_session,
    switch_timer_task,
};

#[tauri::command]
pub async fn get_timer_state(
    timer_repo: State<'_, TimerRepositoryArc>,
) -> Result<TimerState, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    app_get_timer_state(timer_repo_arc)
        .await
        .context("infra::commands::timer_cmd::get_timer_state - Failed to retrieve timer state")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_timer(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    let current_state = current_timer.state();

    if current_state.status() == domain::TimerStatus::Paused {
        // Get the active task ID from the timer
        let task_id = current_timer
            .active_task_id()
            .ok_or("No active task in timer")?;

        // Resume the paused timer
        usecases::timer::resume_timer_session(
            task_id,
            task_repo.inner().clone(),
            timer_repo.inner().clone(),
            event_publisher.inner().clone()
        )
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to resume paused timer")
        .map_err(|e| e.to_string())?;
    } else {
        // Try to get an active task, or any incomplete task for starting
        let active_tasks = task_repo
            .get_active_tasks()
            .await
            .map_err(|e| e.to_string())?;

        let task = if let Some(active_task) = active_tasks.first() {
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
        info!("Started timer, {}", task_id);

        let cmd = StartTimerSessionCmd {
            task_id: Some(task_id),
        };

        start_timer_session(
            task_repo.inner().clone(),
            timer_repo.inner().clone(),
            event_publisher.inner().clone(),
            cmd,
        )
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to execute start timer session")
        .map_err(|e| e.to_string())?;
    }

    app_get_timer_state(timer_repo_arc)
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_timer(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TimerState, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    // Get current timer state to find active task
    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::pause_timer - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    // Get the active task ID from the timer
    let task_id = current_timer
        .active_task_id()
        .ok_or("No active task in timer")?;

    pause_timer_session(
        task_id,
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone()
    )
    .await
    .context("infra::commands::timer_cmd::pause_timer - Failed to toggle pause state")
    .map_err(|e| e.to_string())?;

    app_get_timer_state(timer_repo_arc)
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_timer(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TimerState, String> {
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
        event_publisher.inner().clone()
    )
    .await
    .context("infra::commands::timer_cmd::reset_timer - Failed to reset timer to initial state")
    .map_err(|e| e.to_string())?;

    app_get_timer_state(timer_repo_arc)
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn skip_phase(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    // Get current timer state to find active task
    let current_timer = timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::skip_phase - Failed to get current timer")
        .map_err(|e| e.to_string())?;

    // Get the active task ID from the timer
    let task_id = current_timer
        .active_task_id()
        .ok_or("No active task in timer")?;

    let (_old_phase, new_phase) = skip_timer_phase(
        task_repo.inner().clone(),
        timer_repo.inner().clone(),
        event_publisher.inner().clone(),
        task_id,
    )
    .await
    .context(
        "infra::commands::timer_cmd::skip_phase - Failed to skip to next phase",
    )
    .map_err(|e| e.to_string())?;

    // Send tauri event with new phase information
    app_handle
        .emit(ui_listeners::timer::PHASE_SKIPPED, new_phase)
        .map_err(|e| e.to_string())?;

    app_get_timer_state(timer_repo_arc)
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn switch_timer_task_cmd(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, TimerRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TimerState, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    let task_id_parsed = domain::TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    let cmd = SwitchTimerTaskCmd { task_id: task_id_parsed };

    switch_timer_task(
        timer_repo.inner().clone(),
        task_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .context("infra::commands::timer_cmd::switch_timer_task - Failed to switch timer task")
    .map_err(|e| e.to_string())?;

    app_get_timer_state(timer_repo_arc)
        .await
        .context(
            "infra::commands::timer_cmd - Failed to get updated timer state",
        )
        .map_err(|e| e.to_string())
}
