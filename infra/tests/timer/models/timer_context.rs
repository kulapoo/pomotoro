use domain::TaskRepository;
use infra::adapters::{InMemoryTaskRepository, InMemoryTimerService};
use std::sync::Arc;

/// Timer domain test context
pub struct TimerTestContext {
    pub timer_service: Arc<InMemoryTimerService>,
    pub task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TimerTestContext {
    pub fn new() -> Self {
        let task_defaults = domain::TaskDefaults::default();
        Self {
            timer_service: Arc::new(InMemoryTimerService::new()),
            task_repo: Arc::new(InMemoryTaskRepository::with_default_task(
                &task_defaults,
            )),
        }
    }
}
