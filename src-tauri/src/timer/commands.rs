use tauri::{AppHandle, Emitter, State};

use super::manager::TimerManager;
use super::types::{TimerState, TimerStatus};
use super::notifications::send_phase_notification;
use crate::task::types::{TaskId, Task};
use crate::task::repository::TaskRepository;

#[derive(serde::Serialize)]
pub struct TimerStateWithTask {
    pub timer_state: TimerState,
    pub active_task: Option<Task>,
}

#[tauri::command]
pub async fn get_timer_state(
    timer_manager: State<'_, TimerManager>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let _ = timer_manager.load_state(&app_handle).await;
    
    Ok(timer_manager.get_state().await)
}

#[tauri::command]
pub async fn start_timer(
    timer_manager: State<'_, TimerManager>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let current_state = timer_manager.get_state().await;
    
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
            timer_manager.set_status(TimerStatus::Running).await;
        }
        TimerStatus::Running => return Ok(current_state),
    }
    
    timer_manager.start_timer(app_handle, task).await;
    
    Ok(timer_manager.get_state().await)
}

#[tauri::command]
pub async fn pause_timer(
    timer_manager: State<'_, TimerManager>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    timer_manager.stop_timer().await;
    
    let current_state = timer_manager.get_state().await;
    if current_state.status == TimerStatus::Running {
        timer_manager.set_status(TimerStatus::Paused).await;
    }
    
    let _ = timer_manager.save_state(&app_handle).await;
    Ok(timer_manager.get_state().await)
}

#[tauri::command]
pub async fn reset_timer(
    timer_manager: State<'_, TimerManager>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let current_state = timer_manager.get_state().await;
    
    let task = if let Some(task_id) = current_state.active_task_id {
        task_repository
            .get_by_id(task_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    timer_manager.stop_timer().await;
    timer_manager.reset_current_phase(task.as_ref()).await;
    
    let _ = timer_manager.save_state(&app_handle).await;
    Ok(timer_manager.get_state().await)
}

#[tauri::command]
pub async fn skip_phase(
    timer_manager: State<'_, TimerManager>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerState, String> {
    let current_state = timer_manager.get_state().await;
    
    let task = if let Some(task_id) = current_state.active_task_id {
        task_repository
            .get_by_id(task_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    timer_manager.stop_timer().await;
    
    let (current_phase, new_phase) = timer_manager.skip_to_next_phase(task.as_ref()).await;
    
    send_phase_notification(&app_handle, &current_phase, &new_phase);
    
    let _ = app_handle.emit("phase-complete", (&current_phase, &new_phase));
    
    let _ = timer_manager.save_state(&app_handle).await;
    Ok(timer_manager.get_state().await)
}

#[tauri::command]
pub async fn get_timer_state_with_task(
    timer_manager: State<'_, TimerManager>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerStateWithTask, String> {
    let _ = timer_manager.load_state(&app_handle).await;
    let timer_state = timer_manager.get_state().await;
    
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
    timer_manager: State<'_, TimerManager>,
    task_repository: State<'_, TaskRepository>,
    app_handle: AppHandle,
) -> Result<TimerStateWithTask, String> {
    let task = task_repository
        .get_by_id(task_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Task not found")?;

    timer_manager.switch_task(task_id, Some(&task)).await;
    timer_manager.save_state(&app_handle).await?;

    let timer_state = timer_manager.get_state().await;
    Ok(TimerStateWithTask {
        timer_state,
        active_task: Some(task),
    })
}