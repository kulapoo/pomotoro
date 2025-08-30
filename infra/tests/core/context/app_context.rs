use std::sync::Arc;
use domain::{Result, TaskRepository, ConfigRepository};
use infra::adapters::{
    database::{SqliteTaskRepository, SqliteConfigRepository, SqliteTimerRepository},
    timer::SqliteTimerService,
};
use domain::task::DefaultCyclingService;

use crate::core::{
    database::{TestDatabase, IsolatedDb},
    mocks::{MockEventBus, MockAudioService},
};

/// Application context for integration tests
pub struct AppContext {
    /// Test database instance
    pub db: TestDatabase,
    /// Isolated database operations
    pub isolated_db: IsolatedDb,
    /// Event bus for testing
    pub event_bus: Arc<MockEventBus>,
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
        let isolated_db = IsolatedDb::new(db.pool.clone());
        
        // Create event bus
        let event_bus = Arc::new(MockEventBus::new());
        
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
        
        Ok(Self {
            db,
            isolated_db,
            event_bus,
            task_repo,
            config_repo,
            timer_repo,
            timer_service,
            task_cycling_service,
            audio_service,
        })
    }

    /// Reset the database to a clean state
    pub async fn reset(&self) -> Result<()> {
        self.isolated_db.clear_all_tables()
    }

    /// Clear a specific table
    pub async fn clear_table(&self, table_name: &str) -> Result<()> {
        self.isolated_db.clear_table(table_name)
    }

    /// Get the number of events published
    pub fn event_count(&self) -> usize {
        self.event_bus.published_count()
    }

    /// Check if a specific event type was published
    pub fn has_event(&self, event_type: &str) -> bool {
        self.event_bus.has_event_type(event_type)
    }

    /// Clear all published events
    pub fn clear_events(&self) {
        self.event_bus.clear()
    }

    /// Assert that an event was published
    pub fn assert_event_published(&self, event_type: &str) {
        self.event_bus.assert_event_published(event_type)
    }

    /// Assert that no events were published
    pub fn assert_no_events(&self) {
        self.event_bus.assert_no_events()
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
    use super::*;
    use crate::fixtures::TaskFixtures;

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

    #[tokio::test]
    async fn reset_clears_database() {
        let ctx = AppContext::new().await.unwrap();
        
        // Add a task
        let task = TaskFixtures::simple("To be deleted");
        ctx.task_repo.create(task).await.unwrap();
        
        // Verify it exists
        let tasks = ctx.task_repo.get_all().await.unwrap();
        assert_eq!(tasks.len(), 1);
        
        // Reset
        ctx.reset().await.unwrap();
        
        // Verify it's gone
        let tasks = ctx.task_repo.get_all().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }
}