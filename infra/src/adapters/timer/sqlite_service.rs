use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

use super::timer_dto::SessionHistoryDto;
use crate::adapters::events::mem_event_bus::EventPublisherArc;
use chrono::Utc;
use domain::TimerRepository;
use domain::{
    ConfigRepository, Error, Phase, Result as DomainResult, Task, TaskId,
    TaskRepository, TimerState, TimerStatus,
    timer::{Timer, TimerService},
};

pub struct SqliteTimerService {
    timer: Arc<Mutex<Timer>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    event_publisher: EventPublisherArc,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    session_history: Arc<Mutex<Vec<SessionHistoryDto>>>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl Clone for SqliteTimerService {
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

impl SqliteTimerService {
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

    async fn save_state(&self) -> DomainResult<()> {
        let timer_guard = self.timer.lock().await;
        self.timer_repository
            .save(&*timer_guard)
            .await
            .map_err(|e| Error::RepositoryError {
                message: e.to_string(),
            })
    }

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
                phase: format!("{:?}", phase),
                duration_seconds,
                completed_at: Utc::now(),
                was_skipped,
            };

            let mut history = self.session_history.lock().await;
            history.push(history_entry);

            // Keep only the last 1000 entries
            // Drain oldest entries to maintain memory efficiency
            const MAX_HISTORY_SIZE: usize = 1000;
            const ENTRIES_TO_REMOVE: usize = 100;

            if history.len() > MAX_HISTORY_SIZE {
                history.drain(0..ENTRIES_TO_REMOVE);
            }
        }
    }

    async fn start_timer_internal(
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

        let events = self
            .timer
            .lock()
            .await
            .start(&config)
            .map_err(|e| e.to_string())?;

        self.event_publisher.publish_batch(events);

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
        // Create a cloned service for the timer task
        let service_for_timer = self.clone();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            let mut tick_count = 0u32;

            loop {
                interval.tick().await;

                let should_continue = {
                    let mut timer = timer_clone.lock().await;

                    if !timer.is_running() {
                        false
                    } else {
                        let continue_running = match timer.tick(&config_clone) {
                            Ok((phase_complete, events)) => {
                                // Publish the events
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

                        // Save state every 10 seconds for persistence
                        tick_count += 1;
                        if tick_count % 10 == 0 {
                            // Spawn save operation to avoid blocking the timer
                            let service = service_for_timer.clone();
                            tokio::spawn(async move {
                                if let Err(e) = service.save_state().await {
                                    eprintln!(
                                        "Failed to save timer state: {e}"
                                    );
                                }
                            });
                        }

                        continue_running
                    }
                };

                if !should_continue {
                    break;
                }
            }
        });

        *self.cancel_handle.lock().await = Some(handle);

        // Save initial state after starting
        if let Err(e) = self.save_state().await {
            return Err(e.to_string());
        }
        Ok(())
    }
}

#[async_trait]
impl TimerService for SqliteTimerService {
    async fn get_timer(&self) -> DomainResult<Timer> {
        Ok(self.timer.lock().await.clone())
    }

    async fn get_state(&self) -> DomainResult<TimerState> {
        Ok(self.timer.lock().await.state().clone())
    }

    async fn load_state(&self) -> DomainResult<()> {
        let loaded_timer = self.timer_repository.get().await.map_err(|e| {
            Error::RepositoryError {
                message: e.to_string(),
            }
        })?;

        *self.timer.lock().await = loaded_timer;
        Ok(())
    }

    async fn switch_task(
        &self,
        task_id: TaskId,
        _task: Option<&Task>,
    ) -> DomainResult<()> {
        {
            let mut timer = self.timer.lock().await;
            // Only allow switching when timer is idle
            if !timer.state().is_idle() {
                return Err(Error::ConfigurationError {
                    message: "Cannot switch tasks while timer is running"
                        .to_string(),
                });
            }
            timer.set_active_task(task_id);
        }

        self.save_state().await
    }

    async fn stop_timer(&self) -> DomainResult<()> {
        // Cancel the timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        let config = self.config_repository.get_config().await?.timer;

        let events = {
            let mut timer = self.timer.lock().await;
            timer.reset(&config)?
        };

        self.event_publisher.publish_batch(events);
        self.save_state().await?;
        Ok(())
    }

    async fn start_timer(&self, task: Option<&Task>) -> DomainResult<()> {
        self.start_timer_internal(task).await.map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to start timer: {e}"),
            }
        })
    }

    async fn toggle_pause(&self) -> DomainResult<TimerStatus> {
        let config = self.config_repository.get_config().await?.timer;

        let (status, events) = {
            let mut timer = self.timer.lock().await;
            if timer.is_running() {
                let events = timer.pause(&config)?;
                (TimerStatus::Paused, events)
            } else {
                let events = timer.resume(&config)?;
                (TimerStatus::Running, events)
            }
        };

        self.event_publisher.publish_batch(events);

        // If resumed, restart the timer task
        if status == TimerStatus::Running {
            // Restart the timer task without a specific task context
            self.start_timer_internal(None).await.map_err(|e| {
                Error::RepositoryError {
                    message: format!("Failed to resume timer: {e}"),
                }
            })?;
        }

        self.save_state().await?;
        Ok(status)
    }

    async fn reset_current_phase(
        &self,
        task: Option<&Task>,
    ) -> DomainResult<()> {
        // Cancel the timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        let config = if let Some(task) = task {
            task.config.timer.clone()
        } else {
            self.config_repository.get_config().await?.timer
        };

        let events = {
            let mut timer = self.timer.lock().await;
            timer.reset(&config)?
        };

        self.event_publisher.publish_batch(events);
        self.save_state().await?;
        Ok(())
    }

    async fn skip_to_next_phase(
        &self,
        task: Option<&Task>,
    ) -> DomainResult<(Phase, Phase)> {
        let (old_phase, duration) = {
            let timer = self.timer.lock().await;
            let state = timer.state();
            let phase = state.phase();
            let duration = state.remaining_seconds();
            (phase, duration)
        };

        self.add_session_history(task, old_phase, duration, true)
            .await;

        let config = if let Some(task) = task {
            task.config.timer.clone()
        } else {
            self.config_repository.get_config().await?.timer
        };

        let (events, new_phase) = {
            let mut timer = self.timer.lock().await;
            let events = timer.skip_phase(&config)?;
            let new_phase = timer.state().phase();
            (events, new_phase)
        };

        self.event_publisher.publish_batch(events);
        self.save_state().await?;
        Ok((old_phase, new_phase))
    }
}
