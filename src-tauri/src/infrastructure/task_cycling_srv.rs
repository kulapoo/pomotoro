use async_trait::async_trait;
use pomotoro_domain::{
    DefaultTaskCyclingService as DomainTaskCyclingService, Result, Task, TaskCyclerService,
    TaskCyclingStrategy, TaskId, TaskRepository,
};
use std::sync::Arc;

/// Standard implementation of task cycling service
/// Handles I/O operations and orchestrates domain logic with repository
pub struct StandardTaskCyclerService {
    task_repository: Arc<dyn TaskRepository>,
    domain_service: DomainTaskCyclingService,
    cycling_strategy: TaskCyclingStrategy,
}

impl StandardTaskCyclerService {
    pub fn new(
        task_repository: Arc<dyn TaskRepository>,
        cycling_strategy: TaskCyclingStrategy,
    ) -> Self {
        Self {
            task_repository,
            domain_service: DomainTaskCyclingService::new(),
            cycling_strategy,
        }
    }

    async fn get_available_tasks(&self) -> Result<Vec<Task>> {
        let all_tasks = self.task_repository.get_active_tasks().await?;
        Ok(self.domain_service.filter_available_tasks(&all_tasks))
    }
}

#[async_trait]
impl TaskCyclerService for StandardTaskCyclerService {
    async fn get_next_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let available_tasks = self.get_available_tasks().await?;

        let next_task = self.domain_service.apply_cycling_strategy(
            &self.cycling_strategy,
            &available_tasks,
            current_task_id,
        );

        Ok(next_task.cloned())
    }

    async fn validate_task_switch(&self, task_id: TaskId) -> Result<Option<Task>> {
        let task = self.task_repository.get_by_id(task_id.clone()).await?;

        if let Some(ref task) = task {
            self.domain_service.can_switch_to_task(task)?;
        }

        Ok(task)
    }

    async fn get_active_task_queue(&self) -> Result<Vec<Task>> {
        self.get_available_tasks().await
    }

    async fn cycle_to_next_active_task(
        &self,
        current_task_id: Option<TaskId>,
    ) -> Result<Option<Task>> {
        self.get_next_task(current_task_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{TaskDefaults, Error};
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Test implementation for infrastructure layer testing
    struct TestTaskRepository {
        tasks: Mutex<HashMap<TaskId, Task>>,
    }

    impl TestTaskRepository {
        fn new() -> Self {
            Self {
                tasks: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl TaskRepository for TestTaskRepository {
        async fn create(&self, task: Task) -> Result<()> {
            self.tasks.lock().unwrap().insert(task.id.clone(), task);
            Ok(())
        }

        async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
            Ok(self.tasks.lock().unwrap().get(&id).cloned())
        }

        async fn get_all(&self) -> Result<Vec<Task>> {
            Ok(self.tasks.lock().unwrap().values().cloned().collect())
        }

        async fn get_active_tasks(&self) -> Result<Vec<Task>> {
            Ok(self
                .tasks
                .lock()
                .unwrap()
                .values()
                .filter(|t| {
                    t.status == pomotoro_domain::TaskStatus::Active
                        || t.status == pomotoro_domain::TaskStatus::Queued
                })
                .cloned()
                .collect())
        }

        async fn update(&self, task: Task) -> Result<()> {
            self.tasks.lock().unwrap().insert(task.id.clone(), task);
            Ok(())
        }

        async fn delete(&self, id: TaskId) -> Result<bool> {
            Ok(self.tasks.lock().unwrap().remove(&id).is_some())
        }

        async fn get_by_tags(&self, _tags: &[String]) -> Result<Vec<Task>> {
            Ok(vec![]) // Simplified for tests
        }

        async fn get_by_status(&self, status: pomotoro_domain::TaskStatus) -> Result<Vec<Task>> {
            Ok(self
                .tasks
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.status == status)
                .cloned()
                .collect())
        }

        async fn exists(&self, id: TaskId) -> Result<bool> {
            Ok(self.tasks.lock().unwrap().contains_key(&id))
        }
    }

    async fn setup_service() -> (StandardTaskCyclerService, Arc<TestTaskRepository>) {
        let task_repo = Arc::new(TestTaskRepository::new());
        let service =
            StandardTaskCyclerService::new(task_repo.clone(), TaskCyclingStrategy::RoundRobin);
        (service, task_repo)
    }

    async fn create_test_tasks(repo: &TestTaskRepository) -> Vec<Task> {
        let defaults = TaskDefaults::default();
        let task1 = Task::new_with_defaults("Task 1".to_string(), 4, &defaults).unwrap();
        let task2 = Task::new_with_defaults("Task 2".to_string(), 3, &defaults).unwrap();
        let task3 = Task::new_with_defaults("Task 3".to_string(), 2, &defaults).unwrap();

        repo.create(task1.clone()).await.unwrap();
        repo.create(task2.clone()).await.unwrap();
        repo.create(task3.clone()).await.unwrap();

        vec![task1, task2, task3]
    }

    #[tokio::test]
    async fn should_get_next_task_in_round_robin() {
        let (service, repo) = setup_service().await;
        let _tasks = create_test_tasks(&repo).await;

        // First call should return some task
        let first_task = service.get_next_task(None).await.unwrap().unwrap();
        let first_task_id = first_task.id;

        // Next call with current task should return a different task
        let second_task = service
            .get_next_task(Some(first_task_id.clone()))
            .await
            .unwrap()
            .unwrap();
        assert_ne!(second_task.id, first_task_id);

        // After cycling through all tasks, should return to the first task again
        let third_task = service
            .get_next_task(Some(second_task.id))
            .await
            .unwrap()
            .unwrap();
        let fourth_task = service
            .get_next_task(Some(third_task.id))
            .await
            .unwrap()
            .unwrap();

        // After cycling through all 3 tasks, should get back to first task
        assert_eq!(fourth_task.id, first_task_id);
    }

    #[tokio::test]
    async fn should_switch_to_specific_task() {
        let (service, repo) = setup_service().await;
        let tasks = create_test_tasks(&repo).await;

        let switched_task = service
            .validate_task_switch(tasks[1].id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(switched_task.name, "Task 2");
        assert_eq!(switched_task.id, tasks[1].id);
    }

    #[tokio::test]
    async fn should_fail_to_switch_to_completed_task() {
        let (service, repo) = setup_service().await;
        let defaults = TaskDefaults::default();
        let mut task = Task::new_with_defaults("Completed Task".to_string(), 1, &defaults).unwrap();
        task.increment_session().unwrap(); // Complete the task
        let task_id = task.id.clone();
        repo.create(task).await.unwrap();

        let result = service.validate_task_switch(task_id).await;
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }

    #[tokio::test]
    async fn should_get_active_task_queue() {
        let (service, repo) = setup_service().await;
        let _tasks = create_test_tasks(&repo).await;

        let queue = service.get_active_task_queue().await.unwrap();
        assert_eq!(queue.len(), 3);
        assert!(queue.iter().all(|t| !t.is_completed()));
    }

    #[tokio::test]
    async fn should_exclude_completed_tasks_from_queue() {
        let (service, repo) = setup_service().await;
        let mut tasks = create_test_tasks(&repo).await;

        // Complete one task
        tasks[1].increment_session().unwrap(); // Complete first session
        tasks[1].increment_session().unwrap(); // Complete second session
        tasks[1].increment_session().unwrap(); // Complete all sessions
        repo.update(tasks[1].clone()).await.unwrap();

        let queue = service.get_active_task_queue().await.unwrap();
        assert_eq!(queue.len(), 2);
        assert!(!queue.iter().any(|t| t.name == "Task 2"));
    }

    #[tokio::test]
    async fn should_cycle_to_next_active_task() {
        let (service, repo) = setup_service().await;
        let tasks = create_test_tasks(&repo).await;

        let next_task = service
            .cycle_to_next_active_task(Some(tasks[0].id.clone()))
            .await
            .unwrap()
            .unwrap();
        // Should return a different task than the current one
        assert_ne!(next_task.id, tasks[0].id);
        // Should be one of the available tasks
        assert!(tasks.iter().any(|t| t.id == next_task.id));
    }

    #[tokio::test]
    async fn should_handle_manual_cycling_strategy() {
        let task_repo = Arc::new(TestTaskRepository::new());
        let service =
            StandardTaskCyclerService::new(task_repo.clone(), TaskCyclingStrategy::Manual);

        let tasks = create_test_tasks(&task_repo).await;

        // In manual mode, should return current task, not cycle
        let next_task = service
            .get_next_task(Some(tasks[0].id.clone()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(next_task.id, tasks[0].id);
        assert_eq!(next_task.name, "Task 1");
    }
}
