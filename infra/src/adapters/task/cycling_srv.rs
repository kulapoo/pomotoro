use std::sync::Arc;
use async_trait::async_trait;
use domain::{Result, TaskRepository, TaskId, Task, TaskCyclerService};

pub struct DefaultCyclingService {
    task_repository: Arc<dyn TaskRepository>,
}

impl DefaultCyclingService {
    pub fn new(task_repository: Arc<dyn TaskRepository>) -> Self {
        Self { task_repository }
    }
}

#[async_trait]
impl TaskCyclerService for DefaultCyclingService {

    async fn get_next_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let active_tasks = self.task_repository.get_active_tasks().await?;

        if active_tasks.is_empty() {
            return Ok(None);
        }

        let Some(current_id) = current_task_id else {
            return Ok(active_tasks.into_iter().next());
        };

        let current_index = active_tasks
            .iter()
            .position(|t| t.id() == current_id);

        match current_index {
            Some(idx) => {
                let next_idx = (idx + 1) % active_tasks.len();
                Ok(active_tasks.get(next_idx).cloned())
            }
            None => {
                Ok(active_tasks.first().cloned())
            }
        }
    }

    async fn get_previous_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let active_tasks = self.task_repository.get_active_tasks().await?;

        if active_tasks.is_empty() {
            return Ok(None);
        }

        let Some(current_id) = current_task_id else {
            return Ok(active_tasks.last().cloned());
        };

        let current_index = active_tasks
            .iter()
            .position(|t| t.id() == current_id);

        match current_index {
            Some(idx) => {
                let prev_idx = if idx == 0 {
                    active_tasks.len() - 1
                } else {
                    idx - 1
                };
                Ok(active_tasks.get(prev_idx).cloned())
            }
            None => {
                Ok(active_tasks.last().cloned())
            }
        }
    }

    async fn get_default_task(&self) -> Result<Option<Task>> {
        if let Some(default_task) = self.task_repository.get_default_task().await? {
            return Ok(Some(default_task));
        }

        let active_tasks = self.task_repository.get_active_tasks().await?;
        Ok(active_tasks.first().cloned())
    }

    async fn get_cycle_tasks(&self) -> Result<Vec<Task>> {
        self.task_repository.get_active_tasks().await
    }

    async fn has_tasks(&self) -> Result<bool> {
        let tasks = self.task_repository.get_active_tasks().await?;
        Ok(!tasks.is_empty())
    }
    
    async fn cycle_to_next_active_task(
        &self,
        current_task_id: Option<TaskId>,
    ) -> Result<Option<Task>> {
        self.get_next_task(current_task_id).await
    }
    
    async fn get_active_task_queue(&self) -> Result<Vec<Task>> {
        self.task_repository.get_active_tasks().await
    }
    
    async fn cycle_to_next_incomplete_task(
        &self,
        current_task_id: Option<TaskId>,
    ) -> Result<Option<Task>> {
        let incomplete_tasks = self.task_repository.get_incomplete_tasks().await?;
        
        if incomplete_tasks.is_empty() {
            return Ok(None);
        }
        
        let Some(current_id) = current_task_id else {
            return Ok(incomplete_tasks.into_iter().next());
        };
        
        let current_index = incomplete_tasks
            .iter()
            .position(|t| t.id() == current_id);
        
        match current_index {
            Some(idx) => {
                let next_idx = (idx + 1) % incomplete_tasks.len();
                Ok(incomplete_tasks.get(next_idx).cloned())
            }
            None => {
                Ok(incomplete_tasks.first().cloned())
            }
        }
    }
    
    async fn cycle_to_previous_incomplete_task(
        &self,
        current_task_id: Option<TaskId>,
    ) -> Result<Option<Task>> {
        let incomplete_tasks = self.task_repository.get_incomplete_tasks().await?;
        
        if incomplete_tasks.is_empty() {
            return Ok(None);
        }
        
        let Some(current_id) = current_task_id else {
            return Ok(incomplete_tasks.last().cloned());
        };
        
        let current_index = incomplete_tasks
            .iter()
            .position(|t| t.id() == current_id);
        
        match current_index {
            Some(idx) => {
                let prev_idx = if idx == 0 {
                    incomplete_tasks.len() - 1
                } else {
                    idx - 1
                };
                Ok(incomplete_tasks.get(prev_idx).cloned())
            }
            None => {
                Ok(incomplete_tasks.last().cloned())
            }
        }
    }
    
    async fn get_incomplete_task_queue(&self) -> Result<Vec<Task>> {
        self.task_repository.get_incomplete_tasks().await
    }
    
    async fn get_task_cycle_position(&self, task_id: TaskId) -> Result<(usize, usize)> {
        let active_tasks = self.task_repository.get_active_tasks().await?;
        
        let position = active_tasks
            .iter()
            .position(|t| t.id() == task_id)
            .map(|p| p + 1)
            .unwrap_or(0);
        
        Ok((position, active_tasks.len()))
    }
    
    async fn validate_task_switch(&self, task_id: TaskId) -> Result<()> {
        if let Some(task) = self.task_repository.get_by_id(task_id).await? {
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

        let next = service.get_next_task(Some(task1_id)).await.unwrap();
        assert_eq!(next.unwrap().id(), task2_id);

        let next = service.get_next_task(Some(task2_id)).await.unwrap();
        assert_eq!(next.unwrap().id(), task3_id);

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

        let prev = service.get_previous_task(Some(task1_id)).await.unwrap();
        assert_eq!(prev.unwrap().id(), task3_id);

        let prev = service.get_previous_task(Some(task3_id)).await.unwrap();
        assert_eq!(prev.unwrap().id(), task2_id);

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

        let next = service.get_next_task(Some(unknown_id)).await.unwrap();
        assert_eq!(next.unwrap().id(), task1_id);

        let prev = service.get_previous_task(Some(unknown_id)).await.unwrap();
        assert_eq!(prev.unwrap().id(), task1_id);
    }
}