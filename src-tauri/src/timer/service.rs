use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;

use super::models::{Phase, TimerState, TimerStatus};
use super::notifications::send_phase_notification;
use crate::task::models::Task;

pub struct TimerService {
    state: Arc<RwLock<TimerState>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl TimerService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(TimerState::default())),
            cancel_handle: Arc::new(Mutex::new(None)),
        }
    }
    
    async fn get_state_file_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        
        tokio::fs::create_dir_all(&app_data_dir)
            .await
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
            
        Ok(app_data_dir.join("timer_state.json"))
    }
    
    pub async fn save_state(&self, app_handle: &AppHandle) -> Result<(), String> {
        let state_path = Self::get_state_file_path(app_handle).await?;
        let state = self.state.read().await;
        
        let json = serde_json::to_string_pretty(&*state)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;
            
        tokio::fs::write(state_path, json)
            .await
            .map_err(|e| format!("Failed to write state file: {}", e))?;
            
        Ok(())
    }
    
    pub async fn load_state(&self, app_handle: &AppHandle) -> Result<(), String> {
        let state_path = Self::get_state_file_path(app_handle).await?;
        
        if !state_path.exists() {
            return Ok(());
        }
        
        let json = tokio::fs::read_to_string(state_path)
            .await
            .map_err(|e| format!("Failed to read state file: {}", e))?;
            
        let saved_state: TimerState = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize state: {}", e))?;
            
        let mut state = self.state.write().await;
        *state = saved_state;
        state.status = TimerStatus::Stopped;
        
        Ok(())
    }

    pub async fn start_timer(&self, app_handle: AppHandle, task: Option<Task>) {
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }
        
        let state_clone = Arc::clone(&self.state);
        let cancel_handle_clone = Arc::clone(&self.cancel_handle);
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                let mut state = state_clone.write().await;
                
                if state.status != TimerStatus::Running {
                    break;
                }
                
                if state.remaining_seconds > 0 {
                    state.remaining_seconds -= 1;
                    
                    let _ = app_handle.emit("timer-update", state.clone());
                } else {
                    let current_phase = state.phase.clone();
                    state.next_phase(task.as_ref());
                    state.status = TimerStatus::Stopped;
                    
                    send_phase_notification(&app_handle, &current_phase, &state.phase);
                    
                    let _ = app_handle.emit("phase-complete", (&current_phase, &state.phase));
                    let _ = app_handle.emit("timer-update", state.clone());
                    
                    let state_clone_for_save = state_clone.clone();
                    drop(state);
                    
                    let state_path = match Self::get_state_file_path(&app_handle).await {
                        Ok(path) => path,
                        Err(_) => {
                            break;
                        }
                    };
                    
                    let state_to_save = state_clone_for_save.read().await;
                    if let Ok(json) = serde_json::to_string_pretty(&*state_to_save) {
                        let _ = tokio::fs::write(state_path, json).await;
                    }
                    
                    break;
                }
            }
        });
        
        let mut cancel_guard = cancel_handle_clone.lock().await;
        *cancel_guard = Some(handle);
    }

    pub async fn stop_timer(&self) {
        let mut cancel_guard = self.cancel_handle.lock().await;
        if let Some(handle) = cancel_guard.take() {
            handle.abort();
        }
    }

    pub async fn get_state(&self) -> TimerState {
        let state = self.state.read().await;
        state.clone()
    }

    pub async fn set_status(&self, status: TimerStatus) {
        let mut state = self.state.write().await;
        state.status = status;
    }

    pub async fn reset_current_phase(&self, task: Option<&Task>) {
        let mut state = self.state.write().await;
        state.reset_current_phase(task);
    }

    pub async fn skip_to_next_phase(&self, task: Option<&Task>) -> (Phase, Phase) {
        let mut state = self.state.write().await;
        let current_phase = state.phase.clone();
        state.next_phase(task);
        state.status = TimerStatus::Stopped;
        (current_phase, state.phase.clone())
    }

    pub async fn switch_task(&self, task_id: crate::task::models::TaskId, task: Option<&Task>) {
        let mut state = self.state.write().await;
        state.switch_task(task_id, task);
    }

}