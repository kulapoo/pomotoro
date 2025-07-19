use tauri::{AppHandle, Emitter, State};

use super::service::TimerService;
use super::models::TimerState;
use super::notifications::send_phase_notification;
use crate::core::entities::{TaskId, TimerStatus};
use crate::task::models::Task;
use crate::task::repository::TaskRepository;

#[derive(serde::Serialize)]
pub struct TimerStateWithTask {
    pub timer_state: TimerState,
    pub active_task: Option<Task>,
}

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
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let current_state = timer_service.get_state().await;

    let task = if let Some(task_id) = current_state.active_task_id {
        task_repository
            .get_by_id(task_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    match current_state.status {
        TimerStatus::Stopped | TimerStatus::Paused => {
            timer_service.set_status(TimerStatus::Running).await;
        }
        TimerStatus::Running => return Ok(current_state),
    }

    timer_service.start_timer(app_handle, task).await;

    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn pause_timer(
    timer_service: State<'_, TimerService>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    timer_service.stop_timer().await;

    let current_state = timer_service.get_state().await;
    if current_state.status == TimerStatus::Running {
        timer_service.set_status(TimerStatus::Paused).await;
    }

    let _ = timer_service.save_state(&app_handle).await;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn reset_timer(
    timer_service: State<'_, TimerService>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let current_state = timer_service.get_state().await;

    let task = if let Some(task_id) = current_state.active_task_id {
        task_repository
            .get_by_id(task_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    timer_service.stop_timer().await;
    timer_service.reset_current_phase(task.as_ref()).await;

    let _ = timer_service.save_state(&app_handle).await;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn skip_phase(
    timer_service: State<'_, TimerService>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let current_state = timer_service.get_state().await;

    let task = if let Some(task_id) = current_state.active_task_id {
        task_repository
            .get_by_id(task_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    timer_service.stop_timer().await;

    let (current_phase, new_phase) = timer_service.skip_to_next_phase(task.as_ref()).await;

    send_phase_notification(&app_handle, &current_phase, &new_phase);

    let _ = app_handle.emit("phase-complete", (&current_phase, &new_phase));

    let _ = timer_service.save_state(&app_handle).await;
    Ok(timer_service.get_state().await)
}

#[tauri::command]
pub async fn get_timer_state_with_task(
    timer_service: State<'_, TimerService>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerStateWithTask, String> {
    let _ = timer_service.load_state(&app_handle).await;
    let timer_state = timer_service.get_state().await;

    let active_task = if let Some(task_id) = timer_state.active_task_id {
        task_repository
            .get_by_id(task_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    Ok(TimerStateWithTask {
        timer_state,
        active_task,
    })
}

#[tauri::command]
pub async fn switch_active_task(
    task_id: TaskId,
    timer_service: State<'_, TimerService>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerStateWithTask, String> {
    let task = task_repository
        .get_by_id(task_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Task not found")?;

    timer_service.switch_task(task_id, Some(&task)).await;
    timer_service.save_state(&app_handle).await?;

    let timer_state = timer_service.get_state().await;
    Ok(TimerStateWithTask {
        timer_state,
        active_task: Some(task),
    })
}