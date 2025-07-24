use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;

use super::models::TimerState;
use super::notifications::send_phase_notification;
use pomotoro_domain::{
    Phase, TimerStatus, DefaultPhaseTransitionService, PhaseTransitionService,
    TaskSessionService, TaskSessionServiceInterface, TaskRepository, EventPublisher
};
use crate::task::models::Task;
use crate::events::EventPublisherArc;
use pomotoro_domain::events;

pub struct TimerService {
    state: Arc<RwLock<TimerState>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    phase_service: Arc<dyn PhaseTransitionService>,
    task_session_service: Arc<dyn TaskSessionServiceInterface>,
}

impl TimerService {
    pub fn new_with_services(
        event_publisher: EventPublisherArc,
        task_repository: Arc<dyn TaskRepository>,
    ) -> Self {
        let phase_service = Arc::new(DefaultPhaseTransitionService::new(event_publisher.clone()));
        let task_session_service = Arc::new(TaskSessionService::new(
            task_repository,
            event_publisher,
        ));
        
        Self {
            state: Arc::new(RwLock::new(TimerState::default())),
            cancel_handle: Arc::new(Mutex::new(None)),
            phase_service,
            task_session_service,
        }
    }
    
    pub fn new() -> Self {
        // This is a fallback for backward compatibility
        // In practice, the new_with_services should be used
        Self {
            state: Arc::new(RwLock::new(TimerState::default())),
            cancel_handle: Arc::new(Mutex::new(None)),
            phase_service: Arc::new(DefaultPhaseTransitionService::new(
                Arc::new(pomotoro_domain::NoOpEventPublisher)
            )),
            task_session_service: Arc::new(TaskSessionService::new(
                Arc::new(pomotoro_domain::task::repo::InMemoryTaskRepository::new()),
                Arc::new(pomotoro_domain::NoOpEventPublisher),
            )),
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
        let _ = state.set_status(TimerStatus::Stopped);
        
        Ok(())
    }

    pub async fn start_timer(&self, app_handle: AppHandle, task: Option<Task>) -> Result<(), String> {
        // Start the timer using domain service
        {
            let mut state = self.state.write().await;
            self.phase_service.start_timer(&mut *state)
                .map_err(|e| e.to_string())?;
        }
        
        // Cancel any existing timer
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }
        
        let state_clone = Arc::clone(&self.state);
        let cancel_handle_clone = Arc::clone(&self.cancel_handle);
        let phase_service = Arc::clone(&self.phase_service);
        let task_session_service = Arc::clone(&self.task_session_service);
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                let should_transition = {
                    let mut state = state_clone.write().await;
                    
                    if state.status != TimerStatus::Running {
                        break;
                    }
                    
                    if state.remaining_seconds > 0 {
                        state.remaining_seconds -= 1;
                        let _ = app_handle.emit(events::timer::UPDATE_STATE, state.clone());
                        false
                    } else {
                        true
                    }
                };
                
                if should_transition {
                    let transition_result = {
                        let mut state = state_clone.write().await;
                        if phase_service.can_transition(&*state) {
                            phase_service.transition_to_next_phase(&mut *state)
                        } else {
                            break;
                        }
                    };
                    
                    if let Ok(result) = transition_result {
                        // Handle task session completion if work phase was completed
                        if result.work_session_completed && task.is_some() {
                            let task_ref = task.as_ref().unwrap();
                            let _ = task_session_service
                                .complete_session(&task_ref.id.to_string())
                                .await;
                        }
                        
                        let state = state_clone.read().await;
                        send_phase_notification(&app_handle, &result.old_phase, &result.new_phase);
                        
                        let _ = app_handle.emit(events::timer::PHASE_COMPLETE, (&result.old_phase, &result.new_phase));
                        let _ = app_handle.emit(events::timer::UPDATE_STATE, state.clone());
                    
                        // Save state
                        let state_path = match Self::get_state_file_path(&app_handle).await {
                            Ok(path) => path,
                            Err(_) => break,
                        };
                        
                        if let Ok(json) = serde_json::to_string_pretty(&*state) {
                            let _ = tokio::fs::write(state_path, json).await;
                        }
                        
                        // Stop the timer after phase transition
                        let mut state_mut = state_clone.write().await;
                        let _ = state_mut.set_status(TimerStatus::Stopped);
                        break;
                    }
                }
            }
        });
        
        let mut cancel_guard = cancel_handle_clone.lock().await;
        *cancel_guard = Some(handle);
        
        Ok(())
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

    pub async fn set_status(&self, status: TimerStatus) -> Result<(), String> {
        match status {
            TimerStatus::Running => {
                let mut state = self.state.write().await;
                self.phase_service.start_timer(&mut *state)
                    .map_err(|e| e.to_string())
            }
            TimerStatus::Paused => {
                let mut state = self.state.write().await;
                self.phase_service.pause_timer(&mut *state)
                    .map_err(|e| e.to_string())
            }
            TimerStatus::Stopped => {
                let mut state = self.state.write().await;
                let _ = state.set_status(status);
                Ok(())
            }
        }
    }

    pub async fn reset_current_phase(&self, task: Option<&Task>) -> Result<(), String> {
        let mut state = self.state.write().await;
        self.phase_service.reset_timer(&mut *state)
            .map_err(|e| e.to_string())
    }

    pub async fn skip_to_next_phase(&self, task: Option<&Task>) -> Result<(Phase, Phase), String> {
        let mut state = self.state.write().await;
        
        // Force transition by setting remaining time to 0
        state.remaining_seconds = 0;
        
        let result = self.phase_service.transition_to_next_phase(&mut *state)
            .map_err(|e| e.to_string())?;
            
        // Handle task session completion if work phase was completed
        if result.work_session_completed && task.is_some() {
            let task_ref = task.unwrap();
            let _ = self.task_session_service
                .complete_session(&task_ref.id.to_string())
                .await;
        }
        
        let _ = state.set_status(TimerStatus::Stopped);
        Ok((result.old_phase, result.new_phase))
    }

    pub async fn switch_task(&self, task_id: pomotoro_domain::TaskId, task: Option<&Task>) {
        let mut state = self.state.write().await;
        let _ = state.switch_task(task_id, task);
    }

}