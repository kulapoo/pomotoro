use domain::{
    Result, Task, TaskId, TaskRepository,
    task::{TaskCycling, TaskCyclingExt, PureTaskCycling},
};
use std::sync::Arc;

/// Infrastructure adapter that combines I/O operations with pure domain logic.
///
/// This adapter bridges the gap between:
/// - Async I/O operations (repository calls)
/// - Pure domain logic (TaskCycling trait)
///
/// # Clean Architecture Compliance
/// - Lives in infrastructure layer
/// - Handles all I/O operations
/// - Delegates business logic to pure domain services
/// - Provides async interface for use cases
pub struct TaskCyclingAdapter {
    task_repository: Arc<dyn TaskRepository>,
    domain_cycling: Arc<PureTaskCycling>,
}

impl TaskCyclingAdapter {
    /// Creates a new adapter with the default cycling strategy.
    pub fn new(task_repository: Arc<dyn TaskRepository>) -> Self {
        Self {
            task_repository,
            domain_cycling: Arc::new(PureTaskCycling),
        }
    }

    /// Gets the next task in the cycle.
    ///
    /// Combines I/O operation (fetching tasks) with pure domain logic (cycling).
    pub async fn cycle_next(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        // I/O operation: fetch tasks from repository
        let tasks = self.task_repository.get_active_tasks().await?;

        // Pure domain logic: determine next task
        let next_task = self.domain_cycling.get_next_task(
            &tasks,
            current_task_id.as_ref(),
        );

        Ok(next_task)
    }

    /// Gets the previous task in the cycle.
    pub async fn cycle_previous(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        // I/O operation: fetch tasks from repository
        let tasks = self.task_repository.get_active_tasks().await?;

        // Pure domain logic using extension trait
        let previous_task = self.domain_cycling.get_previous_task(
            &tasks,
            current_task_id.as_ref(),
        );

        Ok(previous_task)
    }

    /// Checks if there are any active tasks.
    pub async fn check_has_tasks(&self) -> Result<bool> {
        // I/O operation: fetch tasks from repository
        let tasks = self.task_repository.get_active_tasks().await?;

        // Pure domain logic: check if any active
        Ok(self.domain_cycling.has_tasks(&tasks))
    }

    /// Gets all active (incomplete) tasks.
    pub async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        // I/O operation: fetch all tasks
        let all_tasks = self.task_repository.get_all().await?;

        // Pure domain logic: filter to active only
        Ok(self.domain_cycling.filter_active_tasks(&all_tasks))
    }

    /// Gets only incomplete tasks.
    pub async fn get_incomplete_tasks(&self) -> Result<Vec<Task>> {
        // I/O operation: fetch incomplete tasks directly
        // Some repositories may optimize this query
        self.task_repository.get_incomplete_tasks().await
    }

    /// Gets the default task (first active task or configured default).
    pub async fn get_default_task(&self) -> Result<Option<Task>> {
        // First try to get explicitly configured default
        if let Some(default_task) = self.task_repository.get_default_task().await? {
            return Ok(Some(default_task));
        }

        // Fall back to first active task
        let active_tasks = self.get_active_tasks().await?;
        Ok(active_tasks.into_iter().next())
    }

    /// Gets the position of a task in the cycle.
    pub async fn get_task_cycle_position(&self, task_id: TaskId) -> Result<(usize, usize)> {
        // I/O operation: fetch tasks
        let tasks = self.task_repository.get_active_tasks().await?;

        // Pure domain logic: calculate position
        Ok(self.domain_cycling.get_task_position(&tasks, &task_id))
    }

    /// Validates if a task can be switched to.
    pub async fn validate_task_switch(&self, task_id: TaskId) -> Result<()> {
        // I/O operation: fetch specific task
        if let Some(task) = self.task_repository.get_by_id(task_id).await? {
            // Domain validation
            if task.is_completed() {
                return Err(domain::Error::TaskAlreadyCompleted);
            }
            Ok(())
        } else {
            Err(domain::Error::TaskNotFound {
                id: task_id.to_string(),
            })
        }
    }

    /// Cycles to the next incomplete task.
    pub async fn cycle_to_next_incomplete(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        // I/O operation: fetch incomplete tasks
        let incomplete_tasks = self.task_repository.get_incomplete_tasks().await?;

        // Pure domain logic: find next in cycle
        let next_task = self.domain_cycling.get_next_task(
            &incomplete_tasks,
            current_task_id.as_ref(),
        );

        Ok(next_task)
    }

    /// Cycles to the previous incomplete task.
    pub async fn cycle_to_previous_incomplete(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        // I/O operation: fetch incomplete tasks
        let incomplete_tasks = self.task_repository.get_incomplete_tasks().await?;

        // Pure domain logic: find previous in cycle
        let previous_task = self.domain_cycling.get_previous_task(
            &incomplete_tasks,
            current_task_id.as_ref(),
        );

        Ok(previous_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::{Task, TaskStatus};
    use std::sync::Mutex;

    struct MockTaskRepository {
        tasks: Mutex<Vec<Task>>,
        default_task_id: Option<TaskId>,
    }

    impl MockTaskRepository {
        fn new(tasks: Vec<Task>) -> Self {
            Self {
                tasks: Mutex::new(tasks),
                default_task_id: None,
            }
        }
    }

    #[async_trait]
    impl TaskRepository for MockTaskRepository {
        async fn create(&self, _task: Task) -> Result<()> {
            unimplemented!()
        }

        async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
            Ok(self.tasks.lock().unwrap()
                .iter()
                .find(|t| t.id == id)
                .cloned())
        }

        async fn get_all(&self) -> Result<Vec<Task>> {
            Ok(self.tasks.lock().unwrap().clone())
        }

        async fn get_active_tasks(&self) -> Result<Vec<Task>> {
            Ok(self.tasks.lock().unwrap()
                .iter()
                .filter(|t| !t.status.is_completed())
                .cloned()
                .collect())
        }

        async fn update(&self, _task: Task) -> Result<()> {
            unimplemented!()
        }

        async fn delete(&self, _id: TaskId) -> Result<bool> {
            unimplemented!()
        }

        async fn get_by_tags(&self, _tags: &[String]) -> Result<Vec<Task>> {
            unimplemented!()
        }

        async fn get_by_status(&self, _status: TaskStatus) -> Result<Vec<Task>> {
            unimplemented!()
        }

        async fn exists(&self, _id: TaskId) -> Result<bool> {
            unimplemented!()
        }

        async fn get_default_task(&self) -> Result<Option<Task>> {
            if let Some(id) = self.default_task_id {
                Ok(self.tasks.lock().unwrap()
                    .iter()
                    .find(|t| t.id == id)
                    .cloned())
            } else {
                Ok(None)
            }
        }

        async fn search(
            &self,
            _options: domain::task::repository::SearchOptions,
        ) -> Result<Vec<Task>> {
            unimplemented!()
        }

        async fn search_fuzzy(&self, _query: &str) -> Result<Vec<Task>> {
            unimplemented!()
        }

        async fn get_incomplete_tasks(&self) -> Result<Vec<Task>> {
            Ok(self.tasks.lock().unwrap()
                .iter()
                .filter(|t| !t.status.is_completed())
                .cloned()
                .collect())
        }

        async fn get_completed_tasks(&self) -> Result<Vec<Task>> {
            Ok(self.tasks.lock().unwrap()
                .iter()
                .filter(|t| t.status.is_completed())
                .cloned()
                .collect())
        }
    }

    fn create_test_task(name: &str) -> Task {
        use domain::TaskBuilder;
        TaskBuilder::with_name_and_sessions(name.to_string(), 4)
            .build()
            .expect("Failed to create test task")
    }

    #[tokio::test]
    async fn test_cycle_next_with_active_tasks() {
        let task1 = create_test_task("Task 1");
        let task2 = create_test_task("Task 2");
        let task1_id = task1.id;

        let repo = Arc::new(MockTaskRepository::new(vec![
            task1.clone(),
            task2.clone(),
        ]));
        let adapter = TaskCyclingAdapter::new(repo);

        let next = adapter.cycle_next(Some(task1_id)).await.unwrap();
        assert_eq!(next.unwrap().name, "Task 2");
    }

    #[tokio::test]
    async fn test_check_has_tasks() {
        let repo_with_tasks = Arc::new(MockTaskRepository::new(vec![
            create_test_task("Task 1"),
        ]));
        let adapter = TaskCyclingAdapter::new(repo_with_tasks);
        assert!(adapter.check_has_tasks().await.unwrap());

        let repo_empty = Arc::new(MockTaskRepository::new(vec![]));
        let adapter_empty = TaskCyclingAdapter::new(repo_empty);
        assert!(!adapter_empty.check_has_tasks().await.unwrap());
    }

    #[tokio::test]
    async fn test_get_default_task_fallback() {
        let task1 = create_test_task("Task 1");
        let repo = Arc::new(MockTaskRepository::new(vec![task1.clone()]));
        let adapter = TaskCyclingAdapter::new(repo);

        let default = adapter.get_default_task().await.unwrap();
        assert_eq!(default.unwrap().name, "Task 1");
    }

    #[tokio::test]
    async fn test_validate_task_switch() {
        let task = create_test_task("Task 1");
        let task_id = task.id;

        let repo = Arc::new(MockTaskRepository::new(vec![task.clone()]));
        let adapter = TaskCyclingAdapter::new(repo.clone());

        // Should succeed for incomplete task
        assert!(adapter.validate_task_switch(task_id).await.is_ok());

        // Should fail for completed task
        let mut completed_task = task.clone();
        completed_task.status = TaskStatus::Completed;
        // Set sessions to match max to truly complete the task
        completed_task.current_sessions = completed_task.max_sessions;
        let repo_completed = Arc::new(MockTaskRepository::new(vec![completed_task.clone()]));
        let adapter_completed = TaskCyclingAdapter::new(repo_completed);

        let validation_result = adapter_completed.validate_task_switch(task_id).await;
        assert!(validation_result.is_err(), "Validation should fail for completed task");

        // Should fail for non-existent task
        let unknown_id = TaskId::new();
        assert!(adapter.validate_task_switch(unknown_id).await.is_err());
    }
}