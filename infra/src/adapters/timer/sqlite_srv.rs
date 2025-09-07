use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

use super::timer_dto::SessionHistoryDto;
use crate::adapters::events::mem_event_bus::EventPublisherArc;
use chrono::Utc;
use domain::TimerRepository;
use domain::{
    ConfigRepository, Error, Phase, Result as DomainResult, Task,
    TaskRepository, Timer,
};

/// Infrastructure service for managing timer tick loops and technical concerns
/// This is NOT a domain service - it handles infrastructure-specific timer management
pub struct TimerTickService {
    timer: Arc<Mutex<Timer>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    event_publisher: EventPublisherArc,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    session_history: Arc<Mutex<Vec<SessionHistoryDto>>>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl Clone for TimerTickService {
    fn clone(&self) -> Self {
        Self {
            timer: Arc::clone(&self.timer),
            cancel_handle: Arc::clone(&self.cancel_handle),
            event_publisher: Arc::clone(&self.event_publisher),
            timer_repository: Arc::clone(&self.timer_repository),
            task_repository: Arc::clone(&self.task_repository),
            session_history: Arc::clone(&self.session_history),
            config_repository: Arc::clone(&self.config_repository),
        }
    }
}

impl TimerTickService {
    pub fn new(
        event_publisher: EventPublisherArc,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        let timer = Timer::default_timer();

        Self {
            timer: Arc::new(Mutex::new(timer)),
            cancel_handle: Arc::new(Mutex::new(None)),
            event_publisher,
            timer_repository,
            task_repository,
            session_history: Arc::new(Mutex::new(Vec::new())),
            config_repository,
        }
    }

    pub async fn save_state(&self) -> DomainResult<()> {
        let timer_guard = self.timer.lock().await;
        self.timer_repository
            .save(&*timer_guard)
            .await
            .map_err(|e| Error::RepositoryError {
                message: e.to_string(),
            })
    }

    // TODO: Remove this once we have a proper session history implementation
    #[allow(dead_code)]
    async fn add_session_history(
        &self,
        task: Option<&Task>,
        phase: Phase,
        duration_seconds: u32,
        was_skipped: bool,
    ) {
        if let Some(task) = task {
            let history_entry = SessionHistoryDto {
                task_id: task.id.to_string(),
                task_name: task.name.clone(),
                phase: phase.name().to_string(),
                duration_seconds,
                completed_at: Utc::now(),
                was_skipped,
            };

            let mut history = self.session_history.lock().await;
            history.push(history_entry);

            // Limit history size to prevent unbounded growth
            const MAX_HISTORY_SIZE: usize = 1000;
            const ENTRIES_TO_REMOVE: usize = 100;

            if history.len() > MAX_HISTORY_SIZE {
                history.drain(0..ENTRIES_TO_REMOVE);
            }
        }
    }

    /// Start the infrastructure timer tick loop
    /// This manages the technical aspects of timer ticking
    pub async fn start_timer_tick_loop(
        &self,
        task: Option<&Task>,
    ) -> Result<(), String> {
        // Get configuration from task or default from config repository
        let config = if let Some(task) = task {
            task.config.timer.clone()
        } else {
            self.config_repository
                .get_config()
                .await
                .map_err(|e| e.to_string())?
                .timer
        };
        // Cancel any existing timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }
        let timer_clone = Arc::clone(&self.timer);
        let event_publisher_clone = Arc::clone(&self.event_publisher);
        let config_clone = config.clone();
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let should_continue = {
                    let mut timer = timer_clone.lock().await;

                    if !timer.is_running() {
                        false
                    } else {
                        let continue_running = match timer.tick(&config_clone) {
                            Ok((phase_complete, events)) => {
                                if !events.is_empty() {
                                    event_publisher_clone.publish_batch(events);
                                }
                                !phase_complete
                            }
                            Err(e) => {
                                eprintln!("Timer tick error: {e}");
                                false
                            }
                        };
                        continue_running
                    }
                };

                if !should_continue {
                    break;
                }
            }
        });

        *self.cancel_handle.lock().await = Some(handle);

        Ok(())
    }

    /// Stop the timer tick loop
    pub async fn stop_timer_tick_loop(&self) -> DomainResult<()> {
        // Cancel the timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }
        Ok(())
    }

    /// Get the current timer for infrastructure purposes
    pub async fn get_current_timer(&self) -> Timer {
        self.timer.lock().await.clone()
    }

    /// Update the timer (for infrastructure use only)
    pub async fn update_timer<F>(&self, update_fn: F) -> DomainResult<()>
    where
        F: FnOnce(&mut Timer) -> DomainResult<()>,
    {
        {
            let mut timer = self.timer.lock().await;
            update_fn(&mut *timer)?;
        }
        self.save_state().await
    }

    /// Load timer state from repository
    pub async fn load_state(&self) -> DomainResult<()> {
        let loaded_timer = self.timer_repository.get().await.map_err(|e| {
            Error::RepositoryError {
                message: e.to_string(),
            }
        })?;
        *self.timer.lock().await = loaded_timer;
        Ok(())
    }

    /// Get session history
    pub async fn get_session_history(&self) -> Vec<SessionHistoryDto> {
        self.session_history.lock().await.clone()
    }
}
