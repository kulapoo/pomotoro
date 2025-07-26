use std::sync::Arc;
use pomotoro_lib::infrastructure::{TimerService, InMemoryTaskRepository};
use pomotoro_domain::TaskRepository;

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

    pub fn with_task_repo(task_repo: Arc<dyn TaskRepository + Send + Sync>) -> Self {
        Self {
            timer_service: Arc::new(TimerService::new()),
            task_repo,
        }
    }
}