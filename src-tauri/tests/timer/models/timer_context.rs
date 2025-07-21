use std::sync::Arc;
use pomotoro_lib::timer::TimerService;
use pomotoro_lib::task::{TaskRepository, InMemoryTaskRepository};

/// Timer domain test context
pub struct TimerTestContext {
    pub timer_service: Arc<TimerService>,
    pub task_repo: TaskRepository,
}

impl TimerTestContext {
    pub fn new() -> Self {
        Self {
            timer_service: Arc::new(TimerService::new()),
            task_repo: Arc::new(InMemoryTaskRepository::with_default_task()),
        }
    }

    pub fn with_task_repo(task_repo: TaskRepository) -> Self {
        Self {
            timer_service: Arc::new(TimerService::new()),
            task_repo,
        }
    }
}