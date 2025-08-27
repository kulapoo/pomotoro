use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

use crate::adapters::events::mem_event_bus::EventPublisherArc;
use super::timer_dto::SessionHistoryDto;
use super::timer_storage_dto::TimerStorageDto;
use domain::{
    ConfigRepository, Error, Phase, Result as DomainResult, Task, TaskId, TimerConfiguration,
    TimerState, TimerStatus, timer::{Timer, TimerService},
};
use chrono::Utc;

pub struct FileTimerService {
    timer: Arc<Mutex<Timer>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    event_publisher: EventPublisherArc,
    data_file: PathBuf,
    session_history: Arc<Mutex<Vec<SessionHistoryDto>>>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl Clone for FileTimerService {
    fn clone(&self) -> Self {
        Self {
            timer: Arc::clone(&self.timer),
            cancel_handle: Arc::clone(&self.cancel_handle),
            event_publisher: Arc::clone(&self.event_publisher),
            data_file: self.data_file.clone(),
            session_history: Arc::clone(&self.session_history),
            config_repository: Arc::clone(&self.config_repository),
        }
    }
}

impl FileTimerService {
    pub fn new(
        event_publisher: EventPublisherArc,
        storage_path: Option<PathBuf>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        let data_file = Self::get_data_file_path(storage_path);
        
        let timer = Timer::new(TimerConfiguration::default())
            .with_event_publisher(Box::new(event_publisher.clone()));

        let service = Self {
            timer: Arc::new(Mutex::new(timer)),
            cancel_handle: Arc::new(Mutex::new(None)),
            event_publisher,
            data_file,
            session_history: Arc::new(Mutex::new(Vec::new())),
            config_repository,
        };
        
        service
    }

    fn get_data_file_path(storage_path: Option<PathBuf>) -> PathBuf {
        let base_path = storage_path.unwrap_or_else(|| {
            dirs::data_dir()
                .expect("Failed to get user data directory")
                .join("pomotoro")
        });
        
        std::fs::create_dir_all(&base_path).ok();
        base_path.join("timer_data.json")
    }

    async fn load_data(&self) -> DomainResult<Option<TimerStorageDto>> {
        if !self.data_file.exists() {
            return Ok(None);
        }

        let json = tokio::fs::read_to_string(&self.data_file)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to read timer data file: {e}"),
            })?;

        let data: TimerStorageDto = serde_json::from_str(&json)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to deserialize timer data: {e}"),
            })?;

        Ok(Some(data))
    }

    async fn save_data(&self) -> DomainResult<()> {
        let timer_guard = self.timer.lock().await;
        let history = self.session_history.lock().await.clone();

        let timer_json = serde_json::to_string(&*timer_guard)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to serialize timer: {e}"),
            })?;
        
        let timer: Timer = serde_json::from_str(&timer_json)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to deserialize timer for save: {e}"),
            })?;

        let data = TimerStorageDto {
            timer,
            last_saved: Utc::now(),
            session_history: history,
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to serialize timer data: {e}"),
            })?;

        let temp_file = self.data_file.with_extension("tmp");
        
        tokio::fs::write(&temp_file, json)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to write timer data file: {e}"),
            })?;

        tokio::fs::rename(&temp_file, &self.data_file)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to rename timer data file: {e}"),
            })?;

        Ok(())
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
            
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }
    }

    async fn start_timer_internal(&self, task: Option<Task>) -> Result<(), String> {
        if let Some(ref task) = task {
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

        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        let timer_clone = Arc::clone(&self.timer);
        let save_data = {
            let service = self.clone();
            move || {
                let service = service.clone();
                async move {
                    service.save_data().await.ok();
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
                        
                        tick_count += 1;
                        if tick_count % 10 == 0 {
                            save_data().await;
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
        self.save_data().await.ok();

        Ok(())
    }

}

#[async_trait]
impl TimerService for FileTimerService {
    async fn get_state(&self) -> DomainResult<TimerState> {
        Ok(self.timer.lock().await.state().clone())
    }

    async fn load_state(&self) -> DomainResult<()> {
        if let Some(data) = self.load_data().await? {
            let mut restored_timer = data.timer;
            restored_timer = restored_timer.with_event_publisher(Box::new(self.event_publisher.clone()));
            
            *self.timer.lock().await = restored_timer;
            *self.session_history.lock().await = data.session_history;
        }
        Ok(())
    }

    async fn switch_task(
        &self,
        task_id: TaskId,
        task: Option<&Task>,
    ) -> DomainResult<()> {
        let mut timer = self.timer.lock().await;

        timer.set_active_entity(Some(task_id.to_string()))?;

        if let Some(task) = task {
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

        drop(timer);
        self.save_data().await?;

        Ok(())
    }

    async fn start_timer(&self, task: Option<&Task>) -> DomainResult<()> {
        self.start_timer_internal(task.cloned())
            .await
            .map_err(|e| Error::RepositoryError { message: e })
    }

    async fn stop_timer(&self) -> DomainResult<()> {
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }

        self.timer.lock().await.reset()?;
        self.save_data().await?;

        Ok(())
    }

    async fn toggle_pause(&self) -> DomainResult<TimerStatus> {
        let mut timer = self.timer.lock().await;

        let new_status = if timer.is_running() {
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

            let timer_clone = Arc::clone(&self.timer);
            let save_data = {
                let service = self.clone();
                move || {
                    let service = service.clone();
                    async move {
                        service.save_data().await.ok();
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
                            let _phase_complete = match timer.tick() {
                                Ok(complete) => complete,
                                Err(e) => {
                                    eprintln!("Timer tick error during resume: {e}");
                                    false
                                }
                            };
                            
                            tick_count += 1;
                            if tick_count % 10 == 0 {
                                save_data().await;
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
            TimerStatus::Running
        } else {
            timer.state().status()
        };

        drop(timer);
        self.save_data().await?;

        Ok(new_status)
    }

    async fn reset_current_phase(&self, _task: Option<&Task>) -> DomainResult<()> {
        self.timer.lock().await.reset()?;
        self.save_data().await?;
        Ok(())
    }

    async fn skip_to_next_phase(
        &self,
        task: Option<&Task>,
    ) -> DomainResult<(Phase, Phase)> {
        let mut timer = self.timer.lock().await;
        let old_phase = timer.state().phase();
        
        let duration = timer.state().configuration().get_phase_duration_seconds(old_phase);
        let remaining = timer.state().remaining_seconds();
        let elapsed = duration - remaining;
        
        timer.skip_phase()?;
        let new_phase = timer.state().phase();

        drop(timer);
        
        self.add_session_history(task, old_phase, elapsed, true).await;
        self.save_data().await?;

        Ok((old_phase, new_phase))
    }
}