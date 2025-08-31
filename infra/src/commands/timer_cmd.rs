use crate::adapters::events::mem_event_bus::EventPublisherArc;
use crate::adapters::TaskRepositoryArc;
use domain::{timer::TimerService, Phase, TaskId, TimerState, event_names::ui_listeners};
use std::sync::Arc;
use tauri::{AppHandle, State, Emitter};
use tracing::info;
use anyhow::Context;

use usecases::timer::{
    StartTimerSessionCmd, SwitchTimerTaskCmd,
    get_timer_state as app_get_timer_state, pause_timer_session,
    reset_timer_session, skip_timer_phase, start_timer_session,
    switch_timer_task,
};

type TimerServiceArc = Arc<dyn TimerService + Send + Sync>;

#[tauri::command]
pub async fn get_timer_state(
    timer_service: State<'_, TimerServiceArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc  = timer_service.inner().clone();

    app_get_timer_state(timer_service_arc.clone())
        .await
        .context("infra::commands::timer_cmd::get_timer_state - Failed to retrieve timer state")
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn start_timer(
    timer_service: State<'_, TimerServiceArc>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc = timer_service.inner().clone();

    let current_state = app_get_timer_state(timer_service_arc.clone())
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to get current timer state")
        .map_err(|e| e.to_string())?;

    if current_state.status() == domain::TimerStatus::Paused {
        pause_timer_session(timer_service_arc.clone(), event_publisher.inner().clone())
            .await
            .context("infra::commands::timer_cmd::start_timer - Failed to resume paused timer")
            .map_err(|e| e.to_string())?;
    } else {
        let task_id = current_state.active_entity_id();
        info!("Started timer, {}", task_id.clone().unwrap_or_default());
        let cmd = StartTimerSessionCmd { task_id };

        start_timer_session(
            timer_service_arc.clone(),
            task_repo.inner().clone(),
            event_publisher.inner().clone(),
            cmd,
        )
        .await
        .context("infra::commands::timer_cmd::start_timer - Failed to execute start timer session")
        .map_err(|e| e.to_string())?;
    }

    app_get_timer_state(timer_service_arc)
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_timer(
    timer_service: State<'_, TimerServiceArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc = timer_service.inner().clone();

    pause_timer_session(timer_service_arc.clone(), event_publisher.inner().clone())
        .await
        .context("infra::commands::timer_cmd::pause_timer - Failed to toggle pause state")
        .map_err(|e| e.to_string())?;

    app_get_timer_state(timer_service_arc)
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_timer(
    timer_service: State<'_, TimerServiceArc>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc = timer_service.inner().clone();

    reset_timer_session(timer_service_arc.clone(), task_repo.inner().clone(), event_publisher.inner().clone())
        .await
        .context("infra::commands::timer_cmd::reset_timer - Failed to reset current phase")
        .map_err(|e| e.to_string())?;

    app_get_timer_state(timer_service_arc)
        .await
        .context("infra::commands::timer_cmd - Failed to get updated timer state")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn skip_phase(
    timer_service: State<'_, TimerServiceArc>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<(Phase, Phase, TimerState), String> {
    let timer_service_arc = timer_service.inner().clone();

    let (old_phase, new_phase) =
        skip_timer_phase(timer_service_arc.clone(), task_repo.inner().clone(), event_publisher.inner().clone())
            .await
            .context("infra::commands::timer_cmd::skip_phase - Failed to skip to next phase")
            .map_err(|e| e.to_string())?;

    let state = usecases::timer::get_timer_state(timer_service_arc)
        .await
        .context("infra::commands::timer_cmd::skip_phase - Failed to get updated timer state")
        .map_err(|e| e.to_string())?;

    Ok((old_phase, new_phase, state))
}

#[tauri::command]
pub async fn switch_active_task(
    task_id: TaskId,
    timer_service: State<'_, TimerServiceArc>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc = timer_service.inner().clone();

    let cmd = SwitchTimerTaskCmd {
        task_id: task_id.to_string(),
    };

    switch_timer_task(timer_service_arc.clone(), task_repo.inner().clone(), event_publisher.inner().clone(), cmd)
        .await
        .with_context(|| format!("Failed to switch to task {}", task_id))
        .map_err(|e| e.to_string())?;

    let updated_state = app_get_timer_state(timer_service_arc.clone())
        .await
        .context("Failed to get updated timer state after task switch")
        .map_err(|e| e.to_string())?;

    // Emit the state update event to notify the UI
    app_handle
        .emit(ui_listeners::timer::STATE_UPDATED, &updated_state)
        .map_err(|e| format!("Failed to emit timer state update: {}", e))?;

    Ok(updated_state)
}
