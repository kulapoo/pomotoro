use tauri::{AppHandle, State};

use crate::infrastructure::{TaskRepositoryArc, TimerService};
use pomotoro_domain::{TaskId, TimerState};
use crate::infrastructure::TimerStateWithTask;

#[tauri::command]
pub async fn get_timer_state(
    timer_service: State<'_, TimerService>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let _ = timer_service.load_state(&app_handle).await;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn start_timer(
    timer_service: State<'_, TimerService>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    timer_service.start_timer(app_handle, None).await?;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn pause_timer(
    timer_service: State<'_, TimerService>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    timer_service.set_status(pomotoro_domain::TimerStatus::Paused).await?;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn reset_timer(
    timer_service: State<'_, TimerService>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    timer_service.reset_current_phase(None).await?;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn skip_phase(
    timer_service: State<'_, TimerService>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    timer_service.skip_to_next_phase(None).await?;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn get_timer_state_with_task(
    timer_service: State<'_, TimerService>,
    task_repo: State<'_, TaskRepositoryArc>,
    app_handle: AppHandle,
) -> Result<TimerStateWithTask, String> {
    let _ = timer_service.load_state(&app_handle).await;
    let state = timer_service.get_state().await;
    
    let task = if let Some(task_id) = &state.active_task_id {
        task_repo.get_by_id(task_id.clone()).await
            .map_err(|e| e.to_string())?
    } else {
        None
    };
    
    Ok(TimerStateWithTask { timer_state: state, active_task: task })
}

#[tauri::command]
pub async fn switch_active_task(
    task_id: TaskId,
    timer_service: State<'_, TimerService>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    timer_service.switch_task(task_id, None).await;
    Ok(timer_service.get_state().await)
}