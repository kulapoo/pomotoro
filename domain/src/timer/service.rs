use crate::{Phase, Result, Task, TaskId, Timer, TimerState, TimerStatus};

/// Timer service trait for application layer orchestration
#[async_trait::async_trait]
pub trait TimerService: Send + Sync {
    async fn get_timer(&self) -> Result<Timer>;
    async fn get_state(&self) -> Result<TimerState>;
    async fn load_state(&self) -> Result<()>;
    async fn switch_task(
        &self,
        task_id: TaskId,
        task: Option<&Task>,
    ) -> Result<()>;
    async fn start_timer(&self, task: Option<&Task>) -> Result<()>;
    async fn stop_timer(&self) -> Result<()>;
    async fn toggle_pause(&self) -> Result<TimerStatus>;
    async fn reset_current_phase(&self, task: Option<&Task>) -> Result<()>;
    async fn skip_to_next_phase(
        &self,
        task: Option<&Task>,
    ) -> Result<(Phase, Phase)>;
}
