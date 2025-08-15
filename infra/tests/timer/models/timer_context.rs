use std::sync::Arc;
use infra::adapters::{TimerService, InMemoryTaskRepository};
use domain::TaskRepository;

/// Timer domain test context
pub struct TimerTestContext {
    pub timer_service: Arc<TimerService>,
    pub task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TimerTestContext {
    pub fn new() -> Self {
        Self {
            timer_service: Arc::new(TimerService::new()),
            task_repo: Arc::new(InMemoryTaskRepository::with_default_task()),
        }
    }

}