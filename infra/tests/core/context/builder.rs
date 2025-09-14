use std::sync::Arc;

use crate::core::{
    context::AppContext,
    database::TestDatabase,
    fixtures::{ConfigFixtures, TaskFixtures},
    mocks::MockAppHandle,
};
use domain::{
    ConfigRepository, EventPublisher, Result, TaskRepository, TimerRepository,
};

/// Builder for customizing app context creation
pub struct AppContextBuilder {
    name: Option<String>,
    with_default_task: bool,
    with_default_config: bool,
    with_task_fixtures: bool,
    with_timer_started: bool,
    task_count: Option<usize>,
    app_handle: Option<MockAppHandle>,
    existing_db: Option<TestDatabase>,
}

impl AppContextBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            with_default_task: false,
            with_default_config: false,
            with_task_fixtures: false,
            with_timer_started: false,
            task_count: None,
            app_handle: None,
            existing_db: None,
        }
    }

    /// Set a custom name for the test database
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add a default task to the context
    pub fn with_default_task(mut self) -> Self {
        self.with_default_task = true;
        self
    }

    /// Add default configuration to the context
    pub fn with_default_config(mut self) -> Self {
        self.with_default_config = true;
        self
    }

    /// Add test task fixtures
    pub fn with_standard_fixtures(mut self) -> Self {
        self.with_default_task = true;
        self.with_default_config = true;
        self.task_count = Some(5);
        self
    }

    pub fn with_task_fixtures(mut self, count: usize) -> Self {
        self.with_task_fixtures = true;
        self.task_count = Some(count);
        self
    }

    pub fn with_app_handle(mut self, app_handle: MockAppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    pub fn with_timer_started(mut self) -> Self {
        self.with_timer_started = true;
        self
    }

    /// Use an existing database (for testing persistence across restarts)
    pub fn with_existing_db(mut self, db: TestDatabase) -> Self {
        self.existing_db = Some(db);
        self
    }

    /// Add test timer fixtures
    /// Build the app context with the specified configuration
    pub async fn build(self) -> Result<AppContext> {
        let ctx = if let Some(db) = self.existing_db {
            AppContext::from_database(db).await?
        } else {
            AppContext::with_name(self.name.as_deref()).await?
        };


        let config = ConfigFixtures::default();
        ctx.config_repo.save_config(&config).await?;



        (*ctx.ui_simulator).clone().start_listen_to_events();
        // Ensure the single timer exists (it should be auto-created by the repository)
        let _ = ctx.timer_repo.get().await?;

        // Add default task if requested
        if self.with_default_task {
            let task = TaskFixtures::with_defaults(
                "Default Task",
                config.timer.sessions_until_long_break,
            );

            ctx.task_repo.create(task).await?;
        }

        // Add task fixtures if requested
        if self.with_task_fixtures {
            let count = self.task_count.unwrap_or(5);
            let tasks = TaskFixtures::collection(count);
            for task in tasks {
                ctx.task_repo.create(task).await?;
            }
        }

        // Add default config if requested



        // Switch to the default task

        if self.with_timer_started {
            let task = ctx
                .task_repo
                .get_default_task()
                .await?
                .ok_or(domain::Error::DefaultTaskNotFound)?;

            ctx.timer_tick_service
                .update_timer(|timer| {
                    timer.set_active_task(task.id);
                    let events =
                        timer.start(&task.config.timer).map_err(|e| {
                            domain::Error::RepositoryError {
                                message: e.to_string(),
                            }
                        })?;

                    (ctx.event_bus.clone() as Arc<dyn EventPublisher>)
                        .publish_batch(events);
                    Ok(())
                })
                .await?;
            ctx.timer_tick_service
                .start_timer_tick_loop(Some(&task))
                .await
                .map_err(|e| domain::Error::RepositoryError { message: e })?;
        }

        Ok(ctx)
    }
}

impl Default for AppContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn builds_empty_context() {
        let ctx = AppContextBuilder::new().build().await.unwrap();

        let tasks = ctx.task_repo.get_all().await.unwrap();
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn builds_context_with_default_task() {
        let ctx = AppContextBuilder::new()
            .with_default_task()
            .build()
            .await
            .unwrap();

        let default_task = ctx.task_repo.get_default_task().await.unwrap();
        assert!(default_task.is_some());
    }

    #[tokio::test]
    async fn builds_context_with_fixtures() {
        let ctx = AppContextBuilder::new()
            .with_task_fixtures(3)
            .build()
            .await
            .unwrap();

        let tasks = ctx.task_repo.get_all().await.unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn builds_context_with_config() {
        let ctx = AppContextBuilder::new()
            .with_default_config()
            .build()
            .await
            .unwrap();

        let config = ctx.config_repo.get_config().await.unwrap();
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn builds_context_with_everything() {
        let ctx = AppContextBuilder::new()
            .with_name("full_test")
            .with_default_task()
            .with_task_fixtures(2)
            .with_default_config()
            .build()
            .await
            .unwrap();

        // Should have 3 tasks total (1 default + 2 fixtures)
        let tasks = ctx.task_repo.get_all().await.unwrap();
        assert_eq!(tasks.len(), 3);

        // Should have config
        let config = ctx.config_repo.get_config().await.unwrap();
        assert!(config.validate().is_ok());

        // Should have custom name in database
        assert!(ctx.db.test_id.starts_with("full_test"));
    }
}
