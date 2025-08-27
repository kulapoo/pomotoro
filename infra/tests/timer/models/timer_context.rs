use domain::{TaskRepository, InMemoryTaskRepository, InMemoryConfigRepository};
use infra::adapters::{FileTimerService, InMemoryEventBus};
use std::sync::Arc;
use tempfile::TempDir;

/// Timer domain test context
pub struct TimerTestContext {
    pub timer_service: Arc<FileTimerService>,
    pub task_repo: Arc<dyn TaskRepository + Send + Sync>,
    _temp_dir: TempDir,
}

impl TimerTestContext {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let event_bus = Arc::new(InMemoryEventBus::new());
        let config_repo = Arc::new(InMemoryConfigRepository::new());
        
        let timer_service = Arc::new(FileTimerService::new(
            event_bus.clone(),
            Some(temp_dir.path().to_path_buf()),
            config_repo,
        ));
        
        Self {
            timer_service,
            task_repo: Arc::new(InMemoryTaskRepository::with_default_task()),
            _temp_dir: temp_dir,
        }
    }
}
