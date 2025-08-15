use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;

use crate::adapters::timer::timer_repo::FileTimerStateRepository;
use crate::adapters::EventPublisherArc;
use domain::timer::TimerService as DomainTimerService;
use domain::Task;
use domain::{
    DefaultPhaseTransitionService, Phase, PhaseTransitionService, TaskId, TimerStatus,
};
use domain::{Result as DomainResult, TimerState};

pub struct TimerService {
    state: Arc<RwLock<TimerState>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    phase_service: Arc<dyn PhaseTransitionService>,
    event_publisher: EventPublisherArc,
    state_repository: Option<FileTimerStateRepository>,
}

impl Clone for TimerService {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            cancel_handle: Arc::clone(&self.cancel_handle),
            phase_service: Arc::clone(&self.phase_service),
            event_publisher: Arc::clone(&self.event_publisher),
            state_repository: None, // TODO: Fix cloning of repository
        }
    }
}

impl Default for TimerService {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerService {
    pub fn new_with_services(
        event_publisher: EventPublisherArc,
        app_handle: Option<AppHandle>,
    ) -> Self {
        let phase_service = Arc::new(DefaultPhaseTransitionService::new());
        let state_repository = app_handle.as_ref().map(|_handle|
            FileTimerStateRepository::new()
        );

        Self {
            state: Arc::new(RwLock::new(TimerState::default())),
            cancel_handle: Arc::new(Mutex::new(None)),
            phase_service,
            event_publisher,
            state_repository,
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
            state_repository: None,
        }
    }

    // Persistence operations moved to FileTimerStateRepository

    pub async fn start_timer_internal(&self, _task: Option<Task>) -> Result<(), String> {
        // Start the timer using domain service
        {
            let mut state = self.state.write().await;
            self.phase_service
                .start_timer(&mut state)
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
                        false
                    } else {
                        true
                    }
                };

                if should_transition {
                    let transition_result = {
                        let mut state = state_clone.write().await;
                        if phase_service.can_transition(&state) {
                            phase_service.transition_to_next_phase(&mut state)
                        } else {
                            break;
                        }
                    };

                    if let Ok(_result) = transition_result {
                        let _state = state_clone.read().await;

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
                    .start_timer(&mut state)
                    .map_err(|e| e.to_string())
            }
            TimerStatus::Paused => {
                let mut state = self.state.write().await;
                self.phase_service
                    .pause_timer(&mut state)
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
            .reset_timer(&mut state)
            .map_err(|e| e.to_string())
    }

    pub async fn skip_to_next_phase(&self, _task: Option<&Task>) -> Result<(Phase, Phase), String> {
        let mut state = self.state.write().await;

        // Force transition by setting remaining time to 0
        state.timer.remaining_seconds = 0;

        let result = self
            .phase_service
            .transition_to_next_phase(&mut state)
            .map_err(|e| e.to_string())?;

        // Note: Business event publishing moved to use cases layer
        // Infrastructure only handles technical timer operations

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
        // Use the internal method that focuses on timer logic
        TimerService::start_timer_internal(self, task.cloned())
            .await
            .map_err(|e| domain::Error::RepositoryError { message: e })
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
        if let Some(ref repository) = self.state_repository {
            if let Some(saved_state) = repository.load_state().await? {
                let mut state = self.state.write().await;
                *state = saved_state;
                // Ensure timer is stopped on load (safety)
                let _ = state.set_status(TimerStatus::Stopped);
            }
        }
        Ok(())
    }

    async fn save_state(&self) -> DomainResult<()> {
        if let Some(ref repository) = self.state_repository {
            let state = self.state.read().await;
            repository.save_state(&state).await?
        }
        Ok(())
    }
}
