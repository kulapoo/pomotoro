use std::sync::{Arc, Mutex};
use pomotoro_lib::task::{TaskRepository};
use pomotoro_lib::config::{ConfigRepository};
use pomotoro_lib::timer::TimerService;
use pomotoro_lib::audio::AudioService;
use super::TestContext;

pub struct TestAppBuilder {
    context: TestContext,
}

impl TestAppBuilder {
    pub fn new() -> Self {
        Self {
            context: TestContext::new(),
        }
    }

    pub fn with_task_repo(task_repo: TaskRepository) -> Self {
        Self {
            context: TestContext::new_with_custom_task_repo(task_repo),
        }
    }

    pub fn build(self) -> TestApp {
        TestApp {
            context: self.context,
        }
    }
}

pub struct TestApp {
    context: TestContext,
}

impl TestApp {
    pub fn task_repo(&self) -> &TaskRepository {
        &self.context.task_repo
    }

    pub fn config_repo(&self) -> &ConfigRepository {
        &self.context.config_repo
    }

    pub fn timer_manager(&self) -> &Arc<TimerService> {
        &self.context.timer_manager
    }

    pub fn audio_manager(&self) -> &Arc<Mutex<AudioService>> {
        &self.context.audio_manager
    }

    pub async fn advance_time(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await;
    }

    pub async fn get_timer_state(&self) -> Result<pomotoro_lib::timer::models::TimerState, String> {
        let timer_manager = self.timer_manager();
        let state = timer_manager.get_state().await;
        Ok(state)
    }

    pub async fn start_timer(&self) -> Result<pomotoro_lib::timer::models::TimerState, String> {
        let timer_manager = self.timer_manager();
        let task_repo = self.task_repo();

        let tasks = task_repo.get_all().await.map_err(|e| e.to_string())?;
        let default_task = tasks.first().ok_or("No default task found")?;

        timer_manager.switch_task(default_task.id, Some(default_task)).await;
        timer_manager.set_status(pomotoro_lib::timer::models::TimerStatus::Running).await;

        Ok(timer_manager.get_state().await)
    }

    pub async fn pause_timer(&self) -> Result<pomotoro_lib::timer::models::TimerState, String> {
        let timer_manager = self.timer_manager();
        timer_manager.set_status(pomotoro_lib::timer::models::TimerStatus::Paused).await;
        Ok(timer_manager.get_state().await)
    }

    pub async fn reset_timer(&self) -> Result<pomotoro_lib::timer::models::TimerState, String> {
        let timer_manager = self.timer_manager();
        let task_repo = self.task_repo();

        let tasks = task_repo.get_all().await.map_err(|e| e.to_string())?;
        let default_task = tasks.first().ok_or("No default task found")?;

        timer_manager.reset_current_phase(Some(default_task)).await;
        timer_manager.set_status(pomotoro_lib::timer::models::TimerStatus::Stopped).await;

        Ok(timer_manager.get_state().await)
    }

    pub async fn create_task(&self, task: pomotoro_lib::task::models::Task) -> Result<(), String> {
        self.task_repo().create(task).await.map_err(|e| e.to_string())
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<pomotoro_lib::task::models::Task>, String> {
        self.task_repo().get_all().await.map_err(|e| e.to_string())
    }

    pub async fn switch_task(&self, task_id: uuid::Uuid) -> Result<pomotoro_lib::timer::models::TimerState, String> {
        let timer_manager = self.timer_manager();
        let task_repo = self.task_repo();

        let task = task_repo.get_by_id(task_id).await.map_err(|e| e.to_string())?;

        timer_manager.switch_task(task_id, task.as_ref()).await;

        Ok(timer_manager.get_state().await)
    }

    pub async fn complete_task_session(&self, task_id: uuid::Uuid) -> Result<pomotoro_lib::task::models::Task, String> {
        let task_repo = self.task_repo();

        let mut task = task_repo.get_by_id(task_id).await
            .map_err(|e| e.to_string())?
            .ok_or("Task not found")?;

        task.increment_session().map_err(|e| e.to_string())?;
        task_repo.update(task.clone()).await.map_err(|e| e.to_string())?;

        Ok(task)
    }

    pub async fn save_global_config(&self, config: pomotoro_lib::config::models::Config) -> Result<(), String> {
        self.config_repo().save_config(&config).map_err(|e| e.to_string())
    }

    pub async fn get_global_config(&self) -> Result<pomotoro_lib::config::models::Config, String> {
        self.config_repo().get_config().map_err(|e| e.to_string())
    }
}

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}