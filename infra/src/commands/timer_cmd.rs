use crate::adapters::events::mem_event_bus::EventPublisherArc;
use crate::adapters::{TaskRepositoryArc, TimerService};
use domain::timer::TimerService as DomainTimerService;
use domain::{Phase, TaskId, TimerState};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tracing::{error, info};
use usecases::timer::{
    StartTimerSessionCmd, SwitchTimerTaskCmd,
    get_timer_state as app_get_timer_state, pause_timer_session,
    reset_timer_session, skip_timer_phase, start_timer_session,
    switch_timer_task,
};

#[tauri::command]
pub async fn get_timer_state(
    timer_service: State<'_, TimerService>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc: Arc<dyn DomainTimerService + Send + Sync> =
        Arc::new(timer_service.inner().clone());

    app_get_timer_state(&timer_service_arc)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_timer(
    timer_service: State<'_, TimerService>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc: Arc<dyn DomainTimerService + Send + Sync> =
        Arc::new(timer_service.inner().clone());

    let cmd = StartTimerSessionCmd { task_id: None };

    match start_timer_session(
        &timer_service_arc,
        &task_repo,
        &event_publisher,
        cmd,
    )
    .await
    {
        Ok(()) => {
            info!("Starting timer session");
        }
        Err(e) => {
            error!("Failed to start timer session: {}", e);
            return Err(e.to_string());
        }
    }

    app_get_timer_state(&timer_service_arc)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_timer(
    timer_service: State<'_, TimerService>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc: Arc<dyn DomainTimerService + Send + Sync> =
        Arc::new(timer_service.inner().clone());

    pause_timer_session(&timer_service_arc, &event_publisher)
        .await
        .map_err(|e| e.to_string())?;

    app_get_timer_state(&timer_service_arc)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_timer(
    timer_service: State<'_, TimerService>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc: Arc<dyn DomainTimerService + Send + Sync> =
        Arc::new(timer_service.inner().clone());

    reset_timer_session(&timer_service_arc, &task_repo, &event_publisher)
        .await
        .map_err(|e| e.to_string())?;

    app_get_timer_state(&timer_service_arc)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn skip_phase(
    timer_service: State<'_, TimerService>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<(Phase, Phase, TimerState), String> {
    let timer_service_arc: Arc<dyn DomainTimerService + Send + Sync> =
        Arc::new(timer_service.inner().clone());

    let (old_phase, new_phase) =
        skip_timer_phase(&timer_service_arc, &task_repo, &event_publisher)
            .await
            .map_err(|e| e.to_string())?;

    let state = usecases::timer::get_timer_state(&timer_service_arc)
        .await
        .map_err(|e| e.to_string())?;

    Ok((old_phase, new_phase, state))
}

#[tauri::command]
pub async fn switch_active_task(
    task_id: TaskId,
    timer_service: State<'_, TimerService>,
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    _app_handle: AppHandle,
) -> Result<TimerState, String> {
    let timer_service_arc: Arc<dyn DomainTimerService + Send + Sync> =
        Arc::new(timer_service.inner().clone());

    let cmd = SwitchTimerTaskCmd {
        task_id: task_id.to_string(),
    };

    switch_timer_task(&timer_service_arc, &task_repo, &event_publisher, cmd)
        .await
        .map_err(|e| e.to_string())?;

    app_get_timer_state(&timer_service_arc)
        .await
        .map_err(|e| e.to_string())
}
