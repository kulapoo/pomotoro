use domain::{Result, TaskRepository, ConfigRepository, timer::TimerService};
use crate::core::{
    context::AppContext,
    fixtures::{ConfigFixtures, TaskFixtures}, mocks::MockAppHandle,
};

/// Builder for customizing app context creation
pub struct AppContextBuilder {
    name: Option<String>,
    with_default_task: bool,
    with_default_config: bool,
    with_task_fixtures: bool,
    task_count: Option<usize>,
    app_handle: Option<MockAppHandle>,
}

impl AppContextBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            with_default_task: false,
            with_default_config: false,
            with_task_fixtures: false,
            task_count: None,
            app_handle: None,
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
        self.with_task_fixtures = true;
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


    /// Add test timer fixtures
    /// Build the app context with the specified configuration
    pub async fn build(self) -> Result<AppContext> {
        let mut ctx = AppContext::with_name(self.name.as_deref()).await?;

        // Add default task if requested
        if self.with_default_task {
            let task = TaskFixtures::default_task();
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
        if self.with_default_config {
            let config = ConfigFixtures::default();
            ctx.config_repo.save_config(&config).await?;
        }

        // Only switch task if a default task exists
        if let Some(task) = ctx.task_repo.get_default_task().await? {
            ctx.timer_service.switch_task(
                task.id(),
                Some(&task)
            ).await?;
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
        let ctx = AppContextBuilder::new()
            .build()
            .await
            .unwrap();

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