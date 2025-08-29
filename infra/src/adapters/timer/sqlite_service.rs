use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

use crate::adapters::events::mem_event_bus::EventPublisherArc;
use crate::adapters::database::TimerRepository;
use super::timer_dto::SessionHistoryDto;
use domain::{
    ConfigRepository, Error, Event, EventPublisher, Phase, Result as DomainResult, Task, TaskId, TimerConfiguration,
    TimerState, TimerStatus, timer::{Timer, TimerService},
};
use chrono::Utc;

/// Wrapper struct that allows an Arc<dyn EventPublisher> to implement EventPublisher
struct ArcEventPublisher {
    inner: EventPublisherArc,
}

impl ArcEventPublisher {
    fn new(inner: EventPublisherArc) -> Self {
        Self { inner }
    }
}

impl EventPublisher for ArcEventPublisher {
    fn publish(&self, event: Box<dyn Event>) {
        self.inner.publish(event);
    }

    fn publish_batch(&self, events: Vec<Box<dyn Event>>) {
        self.inner.publish_batch(events);
    }
}

pub struct SqliteTimerService {
    timer: Arc<Mutex<Timer>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    event_publisher: EventPublisherArc,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
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
            session_history: Arc::clone(&self.session_history),
            config_repository: Arc::clone(&self.config_repository),
        }
    }
}

impl SqliteTimerService {
    pub fn new(
        event_publisher: EventPublisherArc,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        let timer = Timer::new(TimerConfiguration::default())
            .with_event_publisher(Box::new(ArcEventPublisher::new(event_publisher.clone())));

        Self {
            timer: Arc::new(Mutex::new(timer)),
            cancel_handle: Arc::new(Mutex::new(None)),
            event_publisher,
            timer_repository,
            session_history: Arc::new(Mutex::new(Vec::new())),
            config_repository,
        }
    }

    async fn save_state(&self) -> DomainResult<()> {
        let timer_guard = self.timer.lock().await;
        self.timer_repository.save_timer_state(&*timer_guard).await
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
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }
    }

    async fn start_timer_internal(&self, task: Option<&Task>) -> Result<(), String> {
        if let Some(task) = task {
            let config = self.config_repository.get_config().await
                .map_err(|e| e.to_string())?;

            let effective_settings = task.get_effective_settings(&config.task_defaults);

            let timer_config = TimerConfiguration::new(
                effective_settings.work_duration,
                effective_settings.short_break_duration,
                effective_settings.long_break_duration,
                effective_settings.sessions_until_long_break,
            )
            .map_err(|e| e.to_string())?;

            self.timer
                .lock()
                .await
                .update_configuration(timer_config)
                .map_err(|e| e.to_string())?;
        }

        self.timer.lock().await.start().map_err(|e| e.to_string())?;

        // Cancel any existing timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        let timer_clone = Arc::clone(&self.timer);
        let save_state = {
            let service = self.clone();
            move || {
                let service = service.clone();
                async move {
                    service.save_state().await.ok();
                }
            }
        };

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
                        match timer.tick() {
                            Ok(complete) => complete,
                            Err(e) => {
                                eprintln!("Timer tick error: {e}");
                                false
                            }
                        };

                        // Save state every 10 seconds
                        tick_count += 1;
                        if tick_count % 10 == 0 {
                            save_state().await;
                        }

                        true
                    }
                };

                if !should_continue {
                    break;
                }
            }
        });

        *self.cancel_handle.lock().await = Some(handle);
        self.save_state().await.ok();

        Ok(())
    }
}

#[async_trait]
impl TimerService for SqliteTimerService {
    async fn get_state(&self) -> DomainResult<TimerState> {
        Ok(self.timer.lock().await.state().clone())
    }

    async fn load_state(&self) -> DomainResult<()> {
        if let Some(loaded_timer) = self.timer_repository.load_timer_state().await? {
            let mut timer_guard = self.timer.lock().await;
            *timer_guard = loaded_timer
                .with_event_publisher(Box::new(ArcEventPublisher::new(self.event_publisher.clone())));
        }
        Ok(())
    }

    async fn switch_task(
        &self,
        task_id: TaskId,
        task: Option<&Task>,
    ) -> DomainResult<()> {
        let mut timer = self.timer.lock().await;

        // Only allow switching when timer is idle
        if !timer.state().is_idle() {
            return Err(Error::ConfigurationError {
                message: "Cannot switch tasks while timer is running".to_string(),
            });
        }

        // Update the active entity in the timer state
        if let Some(task) = task {
            // Update configuration based on the new task's settings
            let config = self.config_repository.get_config().await?;
            let effective_settings = task.get_effective_settings(&config.task_defaults);

            let timer_config = TimerConfiguration::new(
                effective_settings.work_duration,
                effective_settings.short_break_duration,
                effective_settings.long_break_duration,
                effective_settings.sessions_until_long_break,
            )?;

            timer.update_configuration(timer_config)?;
        }

        // Update the active entity
        timer.set_active_entity(Some(task_id.to_string()))?;

        drop(timer);
        self.save_state().await?;

        Ok(())
    }

    async fn stop_timer(&self) -> DomainResult<()> {
        // Cancel the timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        let mut timer = self.timer.lock().await;
        timer.reset()?;
        drop(timer);

        self.save_state().await?;
        Ok(())
    }

    async fn start_timer(&self, task: Option<&Task>) -> DomainResult<()> {
        self.start_timer_internal(task)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to start timer: {e}"),
            })
    }

    async fn toggle_pause(&self) -> DomainResult<TimerStatus> {
        let mut timer = self.timer.lock().await;
        let status = if timer.is_running() {
            timer.pause()?;
            TimerStatus::Paused
        } else {
            timer.resume()?;
            TimerStatus::Running
        };
        drop(timer);

        // If resumed, restart the timer task
        if status == TimerStatus::Running {
            // We don't have the task here, so just restart without it
            self.start_timer_internal(None)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to resume timer: {e}"),
                })?;
        }

        self.save_state().await?;
        Ok(status)
    }

    async fn reset_current_phase(&self, _task: Option<&Task>) -> DomainResult<()> {
        // Cancel the timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        let mut timer = self.timer.lock().await;
        timer.reset()?;
        drop(timer);

        self.save_state().await?;
        Ok(())
    }

    async fn skip_to_next_phase(&self, task: Option<&Task>) -> DomainResult<(Phase, Phase)> {
        let (old_phase, duration) = {
            let timer = self.timer.lock().await;
            let state = timer.state();
            let phase = state.phase();
            let duration = state.remaining_seconds();
            (phase, duration)
        };

        self.add_session_history(task, old_phase, duration, true).await;

        let mut timer = self.timer.lock().await;
        timer.skip_phase()?;
        let new_phase = timer.state().phase();
        drop(timer);

        self.save_state().await?;
        Ok((old_phase, new_phase))
    }
}