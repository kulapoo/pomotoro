use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio::time::interval;

use crate::adapters::events::mem_event_bus::EventPublisherArc;
use crate::adapters::timer::timer_repo::FileTimerStateRepository;
use domain::{
    Task, Phase, TaskId, TimerStatus, TimerConfiguration,
    timer::Timer,
    Result as DomainResult, TimerState,
};
use usecases::timer::TimerService as DomainTimerService;

pub struct TimerService {
    timer: Arc<Mutex<Timer>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    event_publisher: EventPublisherArc,
    state_repository: Option<Arc<FileTimerStateRepository>>,
}

impl Clone for TimerService {
    fn clone(&self) -> Self {
        Self {
            timer: Arc::clone(&self.timer),
            cancel_handle: Arc::clone(&self.cancel_handle),
            event_publisher: Arc::clone(&self.event_publisher),
            state_repository: self.state_repository.as_ref().map(Arc::clone),
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
        let state_repository = app_handle.as_ref().map(|_handle|
            Arc::new(FileTimerStateRepository::new())
        );

        let timer = Timer::new(TimerConfiguration::default())
            .with_event_publisher(Box::new(event_publisher.clone()));

        Self {
            timer: Arc::new(Mutex::new(timer)),
            cancel_handle: Arc::new(Mutex::new(None)),
            event_publisher,
            state_repository,
        }
    }

    pub fn new() -> Self {
        let event_publisher = Arc::new(domain::NoOpEventPublisher);
        let timer = Timer::new(TimerConfiguration::default())
            .with_event_publisher(Box::new(event_publisher.clone()));
        
        Self {
            timer: Arc::new(Mutex::new(timer)),
            cancel_handle: Arc::new(Mutex::new(None)),
            event_publisher,
            state_repository: None,
        }
    }

    pub async fn get_state(&self) -> DomainResult<TimerState> {
        Ok(self.timer.lock().await.state().clone())
    }

    pub async fn load_state(&self) -> DomainResult<()> {
        if let Some(repo) = &self.state_repository {
            if let Ok(Some(state)) = repo.load_state().await {
                // Create a new timer with the loaded state
                let mut timer = self.timer.lock().await;
                // We need to restore the state - this might require a new method
                // For now, we'll just update configuration
                if let TimerState::Idle { configuration, .. } = &state {
                    timer.update_configuration(configuration.clone())?;
                }
            }
        }
        Ok(())
    }

    pub async fn save_state(&self) -> DomainResult<()> {
        if let Some(repo) = &self.state_repository {
            let state = self.timer.lock().await.state().clone();
            repo.save_state(&state).await?;
        }
        Ok(())
    }

    pub async fn start_timer_internal(&self, task: Option<Task>) -> Result<(), String> {
        // Apply task configuration if provided
        if let Some(ref task) = task {
            let timer_config = TimerConfiguration::new(
                task.config.work_duration(),
                task.config.short_break_duration(),
                task.config.long_break_duration(),
                task.config.sessions_until_long_break(),
            )
            .map_err(|e| e.to_string())?;

            self.timer.lock().await
                .update_configuration(timer_config)
                .map_err(|e| e.to_string())?;
        }

        // Start the timer
        self.timer.lock().await
            .start()
            .map_err(|e| e.to_string())?;

        // Cancel any existing timer
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        // Start the tick loop
        let timer_clone = Arc::clone(&self.timer);
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let should_continue = {
                    let mut timer = timer_clone.lock().await;
                    
                    if !timer.is_running() {
                        false
                    } else {
                        // Process tick
                        let _phase_complete = match timer.tick() {
                            Ok(complete) => complete,
                            Err(e) => {
                                eprintln!("Timer tick error: {e}");
                                false
                            }
                        };
                        true
                    }
                };

                if !should_continue {
                    break;
                }
            }
        });

        *self.cancel_handle.lock().await = Some(handle);
        
        // Save state after starting
        self.save_state().await.ok();
        
        Ok(())
    }

    pub async fn switch_task(&self, task_id: TaskId, task: Option<&Task>) -> DomainResult<()> {
        let mut timer = self.timer.lock().await;
        
        // Set the active task
        timer.set_active_entity(Some(task_id.to_string()))?;
        
        // Update configuration if task provided
        if let Some(task) = task {
            let timer_config = TimerConfiguration::new(
                task.config.work_duration(),
                task.config.short_break_duration(),
                task.config.long_break_duration(),
                task.config.sessions_until_long_break(),
            )?;
            timer.update_configuration(timer_config)?;
        }
        
        drop(timer);
        self.save_state().await.ok();
        
        Ok(())
    }

    pub async fn start_timer(&self, task: Option<&Task>) -> DomainResult<()> {
        self.start_timer_internal(task.cloned())
            .await
            .map_err(|e| domain::Error::RepositoryError { message: e })
    }

    pub async fn stop_timer(&self) -> DomainResult<()> {
        // Cancel the timer loop
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        // Reset the timer
        self.timer.lock().await.reset()?;
        self.save_state().await.ok();
        
        Ok(())
    }

    pub async fn toggle_pause(&self) -> DomainResult<TimerStatus> {
        let mut timer = self.timer.lock().await;
        
        let new_status = if timer.is_running() {
            // Cancel the timer loop when pausing
            {
                let mut cancel_guard = self.cancel_handle.lock().await;
                if let Some(handle) = cancel_guard.take() {
                    handle.abort();
                }
            }
            timer.pause()?;
            TimerStatus::Paused
        } else if timer.is_paused() {
            timer.resume()?;
            
            // Restart the timer loop when resuming
            let timer_clone = Arc::clone(&self.timer);
            let handle = tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(1));

                loop {
                    interval.tick().await;

                    let should_continue = {
                        let mut timer = timer_clone.lock().await;
                        
                        if !timer.is_running() {
                            false
                        } else {
                            let _phase_complete = match timer.tick() {
                                Ok(complete) => complete,
                                Err(e) => {
                                    eprintln!("Timer tick error during resume: {e}");
                                    false
                                }
                            };
                            true
                        }
                    };

                    if !should_continue {
                        break;
                    }
                }
            });

            *self.cancel_handle.lock().await = Some(handle);
            TimerStatus::Running
        } else {
            timer.state().status()
        };
        
        drop(timer);
        self.save_state().await.ok();
        
        Ok(new_status)
    }

    pub async fn reset_current_phase(&self, _task: Option<&Task>) -> DomainResult<()> {
        self.timer.lock().await.reset()?;
        self.save_state().await.ok();
        Ok(())
    }

    pub async fn skip_to_next_phase(&self, _task: Option<&Task>) -> DomainResult<(Phase, Phase)> {
        let mut timer = self.timer.lock().await;
        let old_phase = timer.state().phase();
        timer.skip_phase()?;
        let new_phase = timer.state().phase();
        
        drop(timer);
        self.save_state().await.ok();
        
        Ok((old_phase, new_phase))
    }
}

// Implement the async trait for domain compatibility
#[async_trait]
impl DomainTimerService for TimerService {
    async fn get_state(&self) -> DomainResult<TimerState> {
        self.get_state().await
    }

    async fn load_state(&self) -> DomainResult<()> {
        self.load_state().await
    }

    async fn switch_task(&self, task_id: TaskId, task: Option<&Task>) -> DomainResult<()> {
        self.switch_task(task_id, task).await
    }

    async fn start_timer(&self, task: Option<&Task>) -> DomainResult<()> {
        self.start_timer(task).await
    }

    async fn stop_timer(&self) -> DomainResult<()> {
        self.stop_timer().await
    }

    async fn toggle_pause(&self) -> DomainResult<TimerStatus> {
        self.toggle_pause().await
    }

    async fn reset_current_phase(&self, task: Option<&Task>) -> DomainResult<()> {
        self.reset_current_phase(task).await
    }

    async fn skip_to_next_phase(&self, task: Option<&Task>) -> DomainResult<(Phase, Phase)> {
        self.skip_to_next_phase(task).await
    }
}