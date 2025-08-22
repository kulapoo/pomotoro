use domain::{Task, TaskId, TimerState, TimerStatus, Phase};

/// Timer service trait for application layer orchestration
#[async_trait::async_trait]
pub trait TimerService: Send + Sync {
    async fn get_state(&self) -> domain::Result<TimerState>;
    async fn load_state(&self) -> domain::Result<()>;
    async fn switch_task(&self, task_id: TaskId, task: Option<&Task>) -> domain::Result<()>;
    async fn start_timer(&self, task: Option<&Task>) -> domain::Result<()>;
    async fn stop_timer(&self) -> domain::Result<()>;
    async fn toggle_pause(&self) -> domain::Result<TimerStatus>;
    async fn reset_current_phase(&self, task: Option<&Task>) -> domain::Result<()>;
    async fn skip_to_next_phase(&self, task: Option<&Task>) -> domain::Result<(Phase, Phase)>;
}