use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;

use crate::adapters::notifications::send_phase_notification;
use crate::adapters::EventPublisherArc;
use domain::events::timer;
use domain::timer::TimerService as DomainTimerService;
use domain::Task;
use domain::{
    DefaultPhaseTransitionService, Phase, PhaseTransitionService, TaskId, TimerStatus,
    WorkSessionCompleted,
};
use domain::{Result as DomainResult, TimerState};

pub struct TimerService {
    state: Arc<RwLock<TimerState>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    phase_service: Arc<dyn PhaseTransitionService>,
    event_publisher: EventPublisherArc,
    app_handle: Option<AppHandle>,
}

impl Clone for TimerService {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            cancel_handle: Arc::clone(&self.cancel_handle),
            phase_service: Arc::clone(&self.phase_service),
            event_publisher: Arc::clone(&self.event_publisher),
            app_handle: self.app_handle.clone(),
        }
    }
}

impl TimerService {
    pub fn new_with_services(
        event_publisher: EventPublisherArc,
        app_handle: Option<AppHandle>,
    ) -> Self {
        let phase_service = Arc::new(DefaultPhaseTransitionService::new());

        Self {
            state: Arc::new(RwLock::new(TimerState::default())),
            cancel_handle: Arc::new(Mutex::new(None)),
            phase_service,
            event_publisher,
            app_handle,
        }
    }

    pub fn new() -> Self {
        // This is a fallback for backward compatibility
        // In practice, the new_with_services should be used
        let event_publisher = Arc::new(domain::NoOpEventPublisher);
        Self {
            state: Arc::new(RwLock::new(TimerState::default())),
            cancel_handle: Arc::new(Mutex::new(None)),
            phase_service: Arc::new(DefaultPhaseTransitionService::new()),
            event_publisher,
            app_handle: None,
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

    pub async fn start_timer(
        &self,
        app_handle: AppHandle,
        task: Option<Task>,
    ) -> Result<(), String> {
        // Start the timer using domain service
        {
            let mut state = self.state.write().await;
            self.phase_service
                .start_timer(&mut *state)
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
        let event_publisher = Arc::clone(&self.event_publisher);

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let should_transition = {
                    let mut state = state_clone.write().await;

                    if state.status() != TimerStatus::Running {
                        break;
                    }

                    if state.remaining_seconds() > 0 {
                        state.timer.remaining_seconds =
                            state.timer.remaining_seconds.saturating_sub(1);
                        let _ = app_handle.emit(timer::UPDATE_STATE, state.clone());
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
                        // Publish work session completed event if work phase was completed
                        if result.work_session_completed && task.is_some() {
                            let task_ref = task.as_ref().unwrap();
                            let state = state_clone.read().await;
                            let work_session_event = WorkSessionCompleted::new(
                                Some(task_ref.id.clone()),
                                1500, // 25 minutes work session default duration
                                state.session_count(),
                                task_ref.current_sessions as u32 + 1, // increment since we just completed
                                1,                                    // version
                            );
                            event_publisher.publish(Box::new(work_session_event));
                        }

                        let state = state_clone.read().await;
                        send_phase_notification(&app_handle, &result.old_phase, &result.new_phase);

                        let _ = app_handle.emit(
                            timer::PHASE_COMPLETE,
                            (&result.old_phase, &result.new_phase),
                        );
                        let _ = app_handle.emit(timer::UPDATE_STATE, state.clone());

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
                self.phase_service
                    .start_timer(&mut *state)
                    .map_err(|e| e.to_string())
            }
            TimerStatus::Paused => {
                let mut state = self.state.write().await;
                self.phase_service
                    .pause_timer(&mut *state)
                    .map_err(|e| e.to_string())
            }
            TimerStatus::Stopped => {
                let mut state = self.state.write().await;
                let _ = state.set_status(status);
                Ok(())
            }
        }
    }

    pub async fn reset_current_phase(&self, _task: Option<&Task>) -> Result<(), String> {
        let mut state = self.state.write().await;
        self.phase_service
            .reset_timer(&mut *state)
            .map_err(|e| e.to_string())
    }

    pub async fn skip_to_next_phase(&self, task: Option<&Task>) -> Result<(Phase, Phase), String> {
        let mut state = self.state.write().await;

        // Force transition by setting remaining time to 0
        state.timer.remaining_seconds = 0;

        let result = self
            .phase_service
            .transition_to_next_phase(&mut *state)
            .map_err(|e| e.to_string())?;

        // Publish work session completed event if work phase was completed
        if result.work_session_completed && task.is_some() {
            let task_ref = task.unwrap();
            let work_session_event = WorkSessionCompleted::new(
                Some(task_ref.id.clone()),
                1500, // 25 minutes work session default duration
                state.session_count(),
                task_ref.current_sessions as u32 + 1, // increment since we just completed
                1,                                    // version
            );
            self.event_publisher.publish(Box::new(work_session_event));
        }

        let _ = state.set_status(TimerStatus::Stopped);
        Ok((result.old_phase, result.new_phase))
    }

    pub async fn switch_task(&self, task_id: domain::TaskId, task: Option<&Task>) {
        let mut state = self.state.write().await;
        if let Some(task) = task {
            // Convert TaskConfig to TimerConfiguration
            use domain::TimerConfiguration;
            let timer_config = TimerConfiguration::new(
                task.config.work_duration(),
                task.config.short_break_duration(),
                task.config.long_break_duration(),
                task.config.sessions_until_long_break(),
            )
            .unwrap(); // TaskConfig validation ensures this won't fail

            let _ = state.switch_task_with_config(task_id, timer_config);
        } else {
            let _ = state.switch_task(task_id);
        }
    }
}

/// Implementation of domain TimerService trait
///
/// This allows the infrastructure TimerService to be used through
/// the domain abstraction in the application layer.
#[async_trait]
impl DomainTimerService for TimerService {
    async fn start_timer(&self, task: Option<&Task>) -> DomainResult<()> {
        if let Some(app_handle) = &self.app_handle {
            // Call the existing infrastructure method (different signature)
            TimerService::start_timer(self, app_handle.clone(), task.cloned())
                .await
                .map_err(|e| domain::Error::RepositoryError { message: e })
        } else {
            // Without app handle, we can still start timer but without persistence
            let mut state = self.state.write().await;
            self.phase_service.start_timer(&mut *state).map_err(|e| e)
        }
    }

    async fn stop_timer(&self) -> DomainResult<()> {
        TimerService::stop_timer(self).await;
        Ok(())
    }

    async fn toggle_pause(&self) -> DomainResult<TimerStatus> {
        let current_status = {
            let state = self.state.read().await;
            state.status()
        };

        let new_status = match current_status {
            TimerStatus::Running => {
                TimerService::set_status(self, TimerStatus::Paused)
                    .await
                    .map_err(|e| domain::Error::RepositoryError { message: e })?;
                TimerStatus::Paused
            }
            TimerStatus::Paused => {
                TimerService::set_status(self, TimerStatus::Running)
                    .await
                    .map_err(|e| domain::Error::RepositoryError { message: e })?;
                TimerStatus::Running
            }
            TimerStatus::Stopped => TimerStatus::Stopped,
        };

        Ok(new_status)
    }

    async fn reset_current_phase(&self, task: Option<&Task>) -> DomainResult<()> {
        TimerService::reset_current_phase(self, task)
            .await
            .map_err(|e| domain::Error::RepositoryError { message: e })
    }

    async fn skip_to_next_phase(&self, task: Option<&Task>) -> DomainResult<(Phase, Phase)> {
        TimerService::skip_to_next_phase(self, task)
            .await
            .map_err(|e| domain::Error::RepositoryError { message: e })
    }

    async fn get_state(&self) -> DomainResult<TimerState> {
        Ok(TimerService::get_state(self).await)
    }

    async fn switch_task(&self, task_id: TaskId, task: Option<&Task>) -> DomainResult<()> {
        TimerService::switch_task(self, task_id, task).await;
        Ok(())
    }

    async fn load_state(&self) -> DomainResult<()> {
        if let Some(app_handle) = &self.app_handle {
            TimerService::load_state(self, app_handle)
                .await
                .map_err(|e| domain::Error::RepositoryError { message: e })
        } else {
            Ok(())
        }
    }

    async fn save_state(&self) -> DomainResult<()> {
        if let Some(app_handle) = &self.app_handle {
            TimerService::save_state(self, app_handle)
                .await
                .map_err(|e| domain::Error::RepositoryError { message: e })
        } else {
            Ok(())
        }
    }
}
