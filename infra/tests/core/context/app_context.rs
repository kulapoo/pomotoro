use std::sync::Arc;
use domain::{Result, TaskRepository};
use infra::adapters::{
    database::{SqliteConfigRepository, SqliteTaskRepository, SqliteTimerRepository},
    events::{EventSubscriber, mem_event_bus::InMemoryEventBus},
    task::DefaultCyclingService,
    timer::SqliteTimerService
};

use crate::{core::{database::TestDatabase, mocks::ui::register_test_handlers}, MockAudioService, MockEventBus, UiSimulator};

/// Application context for integration tests
pub struct AppContext {
    /// Test database instance
    pub db: TestDatabase,
    /// Event bus for testing (using real implementation for proper handler execution)
    pub event_bus: Arc<InMemoryEventBus>,
    /// Mock event bus for tracking (used for assertions only)
    pub mock_event_tracker: Arc<MockEventBus>,
    /// Task repository
    pub task_repo: Arc<SqliteTaskRepository>,
    /// Config repository
    pub config_repo: Arc<SqliteConfigRepository>,
    /// Timer repository
    pub timer_repo: Arc<SqliteTimerRepository>,
    /// Timer service
    pub timer_service: Arc<SqliteTimerService>,
    /// Task cycling service
    pub task_cycling_service: Arc<DefaultCyclingService>,
    /// Audio service mock
    pub audio_service: Arc<MockAudioService>,

    pub ui_simulator: Arc<UiSimulator>,
}

impl AppContext {
    /// Create a new app context with isolated database
    pub async fn new() -> Result<Self> {
        Self::with_name(None).await
    }

    /// Create a new app context with a custom test name
    pub async fn with_name(name: Option<&str>) -> Result<Self> {
        // Create isolated test database
        let db = TestDatabase::with_name(name)?;

        // Create real event bus for proper handler execution
        let event_bus = Arc::new(InMemoryEventBus::new());

        // Create mock event tracker for assertions
        let mock_event_tracker = Arc::new(MockEventBus::new());

        // Create repositories
        let task_repo = Arc::new(SqliteTaskRepository::new(db.pool.clone()));
        let config_repo = Arc::new(SqliteConfigRepository::new(db.pool.clone()));
        let timer_repo = Arc::new(SqliteTimerRepository::new(db.pool.clone()));

        // Create services
        let timer_service = Arc::new(SqliteTimerService::new(
            event_bus.clone(),
            timer_repo.clone(),
            config_repo.clone(),
        ));

        let task_cycling_service = Arc::new(DefaultCyclingService::new(
            task_repo.clone(),
        ));

        let audio_service = Arc::new(MockAudioService::new());


        let ui_simulator = Arc::new(
            UiSimulator::new()
        );

        let app_handle = ui_simulator.app_handle().clone();

        register_test_handlers(event_bus.clone() as Arc<dyn EventSubscriber + Send + Sync>, app_handle).unwrap();

        Ok(Self {
            db,
            event_bus,
            mock_event_tracker,
            task_repo,
            config_repo,
            timer_repo,
            timer_service,
            task_cycling_service,
            audio_service,
            ui_simulator,
        })
    }

    /// Get the number of events published
    pub fn event_count(&self) -> usize {
        self.mock_event_tracker.published_count()
    }

    /// Check if a specific event type was published
    pub fn has_event(&self, event_type: &str) -> bool {
        self.mock_event_tracker.has_event_type(event_type)
    }

    /// Clear all published events
    pub fn clear_events(&self) {
        self.mock_event_tracker.clear()
    }

    /// Assert that an event was published
    pub fn assert_event_published(&self, event_type: &str) {
        self.mock_event_tracker.assert_event_published(event_type)
    }

    /// Assert that no events were published
    pub fn assert_no_events(&self) {
        self.mock_event_tracker.assert_no_events()
    }

    /// Get audio service play count
    pub fn audio_play_count(&self) -> usize {
        self.audio_service.play_count()
    }

    /// Reset audio service counts
    pub fn reset_audio_counts(&self) {
        self.audio_service.reset_counts()
    }
}

#[cfg(test)]
mod tests {
    use crate::TaskFixtures;

    use super::*;

    #[tokio::test]
    async fn creates_isolated_context() {
        let ctx = AppContext::new().await.unwrap();
        assert!(ctx.db.exists());
    }

    #[tokio::test]
    async fn repositories_work() {
        let ctx = AppContext::new().await.unwrap();

        // Create a task
        let task = TaskFixtures::simple("Test Task");
        let task_id = task.id();
        ctx.task_repo.create(task).await.unwrap();

        // Retrieve the task
        let retrieved = ctx.task_repo.get_by_id(task_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Task");
    }

}