use std::sync::Arc;
use domain::{Result, TaskRepository, TaskId, Task};

/// Simple task cycling service for rotating between tasks
/// Keeps it simple and stupid (KISS) - just cycles through active tasks
pub struct DefaultCyclingService {
    task_repository: Arc<dyn TaskRepository>,
}

impl DefaultCyclingService {
    pub fn new(task_repository: Arc<dyn TaskRepository>) -> Self {
        Self { task_repository }
    }

    /// Get the next task in the cycle after the given task
    /// If no current task is provided, returns the first active task
    pub async fn get_next_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let active_tasks = self.task_repository.get_active_tasks().await?;
        
        if active_tasks.is_empty() {
            return Ok(None);
        }

        // If no current task, return the first one
        let Some(current_id) = current_task_id else {
            return Ok(active_tasks.into_iter().next());
        };

        // Find current task index
        let current_index = active_tasks
            .iter()
            .position(|t| t.id() == current_id);

        match current_index {
            Some(idx) => {
                // Get next task, wrapping around to the beginning if needed
                let next_idx = (idx + 1) % active_tasks.len();
                Ok(active_tasks.get(next_idx).cloned())
            }
            None => {
                // Current task not found in active tasks, return first active task
                Ok(active_tasks.first().cloned())
            }
        }
    }

    /// Get the previous task in the cycle before the given task
    pub async fn get_previous_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let active_tasks = self.task_repository.get_active_tasks().await?;
        
        if active_tasks.is_empty() {
            return Ok(None);
        }

        // If no current task, return the last one
        let Some(current_id) = current_task_id else {
            return Ok(active_tasks.last().cloned());
        };

        // Find current task index
        let current_index = active_tasks
            .iter()
            .position(|t| t.id() == current_id);

        match current_index {
            Some(idx) => {
                // Get previous task, wrapping around to the end if needed
                let prev_idx = if idx == 0 {
                    active_tasks.len() - 1
                } else {
                    idx - 1
                };
                Ok(active_tasks.get(prev_idx).cloned())
            }
            None => {
                // Current task not found in active tasks, return last active task
                Ok(active_tasks.last().cloned())
            }
        }
    }

    /// Get the default task to start with (first active task or default marked task)
    pub async fn get_default_task(&self) -> Result<Option<Task>> {
        // First try to get explicitly marked default task
        if let Some(default_task) = self.task_repository.get_default_task().await? {
            return Ok(Some(default_task));
        }

        // Otherwise return first active task
        let active_tasks = self.task_repository.get_active_tasks().await?;
        Ok(active_tasks.first().cloned())
    }

    /// Get all active tasks in cycling order
    pub async fn get_cycle_tasks(&self) -> Result<Vec<Task>> {
        self.task_repository.get_active_tasks().await
    }

    /// Check if we have any tasks to cycle through
    pub async fn has_tasks(&self) -> Result<bool> {
        let tasks = self.task_repository.get_active_tasks().await?;
        Ok(!tasks.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use domain::{Task, TaskStatus};

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

        fn with_default(mut self, task_id: TaskId) -> Self {
            self.default_task_id = Some(task_id);
            self
        }
    }

    #[async_trait]
    impl TaskRepository for MockTaskRepository {
        async fn create(&self, _task: Task) -> Result<()> {
            unimplemented!()
        }

        async fn get_by_id(&self, _id: TaskId) -> Result<Option<Task>> {
            unimplemented!()
        }

        async fn get_all(&self) -> Result<Vec<Task>> {
            Ok(self.tasks.lock().unwrap().clone())
        }

        async fn get_active_tasks(&self) -> Result<Vec<Task>> {
            Ok(self.tasks.lock().unwrap().clone())
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
                Ok(self.tasks.lock().unwrap().iter().find(|t| t.id() == id).cloned())
            } else {
                Ok(None)
            }
        }

        async fn search(&self, _options: domain::task::repository::SearchOptions) -> Result<Vec<Task>> {
            unimplemented!()
        }

        async fn search_fuzzy(&self, _query: &str) -> Result<Vec<Task>> {
            unimplemented!()
        }

        async fn get_incomplete_tasks(&self) -> Result<Vec<Task>> {
            unimplemented!()
        }

        async fn get_completed_tasks(&self) -> Result<Vec<Task>> {
            unimplemented!()
        }
    }

    fn create_test_task(_id: &str, name: &str) -> Task {
        Task::new(name.to_string(), 4).expect("Failed to create test task")
    }

    #[tokio::test]
    async fn test_get_next_task_with_empty_list() {
        let repo = Arc::new(MockTaskRepository::new(vec![]));
        let service = DefaultCyclingService::new(repo);
        
        let result = service.get_next_task(None).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_next_task_cycles_correctly() {
        let task1 = create_test_task("1", "Task 1");
        let task2 = create_test_task("2", "Task 2");
        let task3 = create_test_task("3", "Task 3");
        let task1_id = task1.id();
        let task2_id = task2.id();
        let task3_id = task3.id();
        
        let repo = Arc::new(MockTaskRepository::new(vec![
            task1.clone(),
            task2.clone(),
            task3.clone(),
        ]));
        let service = DefaultCyclingService::new(repo);
        
        // Starting from task1 should give task2
        let next = service.get_next_task(Some(task1_id)).await.unwrap();
        assert_eq!(next.unwrap().id(), task2_id);
        
        // Starting from task2 should give task3
        let next = service.get_next_task(Some(task2_id)).await.unwrap();
        assert_eq!(next.unwrap().id(), task3_id);
        
        // Starting from task3 should wrap to task1
        let next = service.get_next_task(Some(task3_id)).await.unwrap();
        assert_eq!(next.unwrap().id(), task1_id);
    }

    #[tokio::test]
    async fn test_get_next_task_with_no_current() {
        let task1 = create_test_task("1", "Task 1");
        let task1_id = task1.id();
        
        let repo = Arc::new(MockTaskRepository::new(vec![task1.clone()]));
        let service = DefaultCyclingService::new(repo);
        
        let next = service.get_next_task(None).await.unwrap();
        assert_eq!(next.unwrap().id(), task1_id);
    }

    #[tokio::test]
    async fn test_get_previous_task_cycles_correctly() {
        let task1 = create_test_task("1", "Task 1");
        let task2 = create_test_task("2", "Task 2");
        let task3 = create_test_task("3", "Task 3");
        let task1_id = task1.id();
        let task2_id = task2.id();
        let task3_id = task3.id();
        
        let repo = Arc::new(MockTaskRepository::new(vec![
            task1.clone(),
            task2.clone(),
            task3.clone(),
        ]));
        let service = DefaultCyclingService::new(repo);
        
        // Starting from task1 should wrap to task3
        let prev = service.get_previous_task(Some(task1_id)).await.unwrap();
        assert_eq!(prev.unwrap().id(), task3_id);
        
        // Starting from task3 should give task2
        let prev = service.get_previous_task(Some(task3_id)).await.unwrap();
        assert_eq!(prev.unwrap().id(), task2_id);
        
        // Starting from task2 should give task1
        let prev = service.get_previous_task(Some(task2_id)).await.unwrap();
        assert_eq!(prev.unwrap().id(), task1_id);
    }

    #[tokio::test]
    async fn test_get_previous_task_with_no_current() {
        let task1 = create_test_task("1", "Task 1");
        let task2 = create_test_task("2", "Task 2");
        let task2_id = task2.id();
        
        let repo = Arc::new(MockTaskRepository::new(vec![task1, task2.clone()]));
        let service = DefaultCyclingService::new(repo);
        
        // Should return the last task
        let prev = service.get_previous_task(None).await.unwrap();
        assert_eq!(prev.unwrap().id(), task2_id);
    }

    #[tokio::test]
    async fn test_get_default_task_with_explicit_default() {
        let task1 = create_test_task("1", "Task 1");
        let task2 = create_test_task("2", "Task 2");
        let task2_id = task2.id();
        
        let repo = Arc::new(
            MockTaskRepository::new(vec![task1, task2.clone()])
                .with_default(task2_id)
        );
        let service = DefaultCyclingService::new(repo);
        
        let default = service.get_default_task().await.unwrap();
        assert_eq!(default.unwrap().id(), task2_id);
    }

    #[tokio::test]
    async fn test_get_default_task_fallback_to_first() {
        let task1 = create_test_task("1", "Task 1");
        let task1_id = task1.id();
        
        let repo = Arc::new(MockTaskRepository::new(vec![task1.clone()]));
        let service = DefaultCyclingService::new(repo);
        
        let default = service.get_default_task().await.unwrap();
        assert_eq!(default.unwrap().id(), task1_id);
    }

    #[tokio::test]
    async fn test_has_tasks() {
        let repo_with_tasks = Arc::new(MockTaskRepository::new(vec![
            create_test_task("1", "Task 1"),
        ]));
        let service_with_tasks = DefaultCyclingService::new(repo_with_tasks);
        assert!(service_with_tasks.has_tasks().await.unwrap());
        
        let repo_empty = Arc::new(MockTaskRepository::new(vec![]));
        let service_empty = DefaultCyclingService::new(repo_empty);
        assert!(!service_empty.has_tasks().await.unwrap());
    }

    #[tokio::test]
    async fn test_task_not_found_fallback() {
        let task1 = create_test_task("1", "Task 1");
        let task1_id = task1.id();
        let unknown_id = TaskId::new();
        
        let repo = Arc::new(MockTaskRepository::new(vec![task1.clone()]));
        let service = DefaultCyclingService::new(repo);
        
        // Unknown task ID should fallback to first task for next
        let next = service.get_next_task(Some(unknown_id)).await.unwrap();
        assert_eq!(next.unwrap().id(), task1_id);
        
        // Unknown task ID should fallback to last task for previous
        let prev = service.get_previous_task(Some(unknown_id)).await.unwrap();
        assert_eq!(prev.unwrap().id(), task1_id);
    }
}