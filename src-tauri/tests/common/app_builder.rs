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

    /// Helper method to simulate time passing for timer tests
    pub async fn advance_time(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await;
    }

    /// Helper method to get current timer state
    pub async fn get_timer_state(&self) -> Result<pomotoro_lib::timer::types::TimerState, String> {
        let timer_manager = self.timer_manager();
        let state = timer_manager.get_state().await;
        Ok(state)
    }

    /// Helper method to start timer with default task
    pub async fn start_timer(&self) -> Result<pomotoro_lib::timer::types::TimerState, String> {
        let timer_manager = self.timer_manager();
        let task_repo = self.task_repo();

        // Get default task
        let tasks = task_repo.get_all().await.map_err(|e| e.to_string())?;
        let default_task = tasks.first().ok_or("No default task found")?;

        // Switch to the task and set status to running
        timer_manager.switch_task(default_task.id, Some(default_task)).await;
        timer_manager.set_status(pomotoro_lib::timer::types::TimerStatus::Running).await;

        Ok(timer_manager.get_state().await)
    }

    /// Helper method to pause timer
    pub async fn pause_timer(&self) -> Result<pomotoro_lib::timer::types::TimerState, String> {
        let timer_manager = self.timer_manager();
        timer_manager.set_status(pomotoro_lib::timer::types::TimerStatus::Paused).await;
        Ok(timer_manager.get_state().await)
    }

    /// Helper method to reset timer
    pub async fn reset_timer(&self) -> Result<pomotoro_lib::timer::types::TimerState, String> {
        let timer_manager = self.timer_manager();
        let task_repo = self.task_repo();

        // Get default task
        let tasks = task_repo.get_all().await.map_err(|e| e.to_string())?;
        let default_task = tasks.first().ok_or("No default task found")?;

        timer_manager.reset_current_phase(Some(default_task)).await;
        timer_manager.set_status(pomotoro_lib::timer::types::TimerStatus::Stopped).await;

        Ok(timer_manager.get_state().await)
    }

    /// Helper method to create a task via the repository
    pub async fn create_task(&self, task: pomotoro_lib::task::types::Task) -> Result<(), String> {
        self.task_repo().create(task).await.map_err(|e| e.to_string())
    }

    /// Helper method to get all tasks
    pub async fn get_all_tasks(&self) -> Result<Vec<pomotoro_lib::task::types::Task>, String> {
        self.task_repo().get_all().await.map_err(|e| e.to_string())
    }

    /// Helper method to switch active task
    pub async fn switch_task(&self, task_id: uuid::Uuid) -> Result<pomotoro_lib::timer::types::TimerState, String> {
        let timer_manager = self.timer_manager();
        let task_repo = self.task_repo();

        // Get the task to switch to
        let task = task_repo.get_by_id(task_id).await.map_err(|e| e.to_string())?;

        timer_manager.switch_task(task_id, task.as_ref()).await;

        Ok(timer_manager.get_state().await)
    }

    /// Helper method to complete a task session
    pub async fn complete_task_session(&self, task_id: uuid::Uuid) -> Result<pomotoro_lib::task::types::Task, String> {
        let task_repo = self.task_repo();

        let mut task = task_repo.get_by_id(task_id).await
            .map_err(|e| e.to_string())?
            .ok_or("Task not found")?;

        task.increment_session();
        task_repo.update(task.clone()).await.map_err(|e| e.to_string())?;

        Ok(task)
    }

    /// Helper method to save global config
    pub async fn save_global_config(&self, config: pomotoro_lib::config::types::GlobalConfig) -> Result<(), String> {
        self.config_repo().save_config(&config).map_err(|e| e.to_string())
    }

    /// Helper method to get global config
    pub async fn get_global_config(&self) -> Result<pomotoro_lib::config::types::GlobalConfig, String> {
        self.config_repo().get_config().map_err(|e| e.to_string())
    }
}

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}