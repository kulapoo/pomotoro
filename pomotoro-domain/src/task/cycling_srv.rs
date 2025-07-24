use crate::{Task, TaskRepository, EventPublisher, ActiveTaskSwitched, Result, Error, TaskId};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait TaskCyclingService: Send + Sync {
    async fn get_next_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>>;
    async fn switch_to_task(&self, task_id: TaskId) -> Result<Option<Task>>;
    async fn get_active_task_queue(&self) -> Result<Vec<Task>>;
    async fn cycle_to_next_active_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>>;
}

#[derive(Debug, Clone)]
pub enum TaskCyclingStrategy {
    Manual,
    RoundRobin,
    PriorityBased,
}

pub struct DefaultTaskCyclingService {
    task_repository: Arc<dyn TaskRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    cycling_strategy: TaskCyclingStrategy,
}

impl DefaultTaskCyclingService {
    pub fn new(
        task_repository: Arc<dyn TaskRepository>,
        event_publisher: Arc<dyn EventPublisher>,
        cycling_strategy: TaskCyclingStrategy,
    ) -> Self {
        Self {
            task_repository,
            event_publisher,
            cycling_strategy,
        }
    }

    fn publish_task_switched(&self, old_task_id: Option<TaskId>, new_task_id: Option<TaskId>) {
        let event = ActiveTaskSwitched::new(
            old_task_id,
            new_task_id,
            crate::Phase::Work, // Default to work phase when switching
            1,
        );

        self.event_publisher.publish(Box::new(event));
    }

    async fn get_available_tasks(&self) -> Result<Vec<Task>> {
        let all_tasks = self.task_repository.get_active_tasks().await?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| !task.is_completed())
            .collect())
    }

    fn find_next_task_round_robin<'a>(&self, tasks: &'a [Task], current_task_id: Option<TaskId>) -> Option<&'a Task> {
        if tasks.is_empty() {
            return None;
        }

        if let Some(current_id) = current_task_id {
            // Find current task position and get next one
            if let Some(current_pos) = tasks.iter().position(|t| t.id == current_id) {
                let next_pos = (current_pos + 1) % tasks.len();
                return Some(&tasks[next_pos]);
            }
        }

        // Return first task if no current task or current task not found
        tasks.first()
    }
}

#[async_trait]
impl TaskCyclingService for DefaultTaskCyclingService {
    async fn get_next_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let available_tasks = self.get_available_tasks().await?;

        let next_task = match self.cycling_strategy {
            TaskCyclingStrategy::Manual => {
                // In manual mode, don't automatically cycle
                current_task_id
                    .and_then(|id| available_tasks.iter().find(|t| t.id == id))
                    .cloned()
            }
            TaskCyclingStrategy::RoundRobin => {
                self.find_next_task_round_robin(&available_tasks, current_task_id)
                    .cloned()
            }
            TaskCyclingStrategy::PriorityBased => {
                // For now, treat priority-based as round-robin
                // TODO: Implement priority logic when Task has priority field
                self.find_next_task_round_robin(&available_tasks, current_task_id)
                    .cloned()
            }
        };

        Ok(next_task)
    }

    async fn switch_to_task(&self, task_id: TaskId) -> Result<Option<Task>> {
        let task = self.task_repository.get_by_id(task_id.clone()).await?;
        
        if let Some(ref task) = task {
            if task.is_completed() {
                return Err(Error::TaskAlreadyCompleted);
            }
        }

        self.publish_task_switched(None, Some(task_id));
        Ok(task)
    }

    async fn get_active_task_queue(&self) -> Result<Vec<Task>> {
        self.get_available_tasks().await
    }

    async fn cycle_to_next_active_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let next_task = self.get_next_task(current_task_id.clone()).await?;
        
        if let Some(ref task) = next_task {
            self.publish_task_switched(current_task_id, Some(task.id.clone()));
        }

        Ok(next_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::repo::InMemoryTaskRepository;

    async fn setup_service() -> (DefaultTaskCyclingService, Arc<InMemoryTaskRepository>) {
        let task_repo = Arc::new(InMemoryTaskRepository::new());
        let event_publisher = Arc::new(crate::NoOpEventPublisher);
        let service = DefaultTaskCyclingService::new(
            task_repo.clone(),
            event_publisher,
            TaskCyclingStrategy::RoundRobin,
        );
        (service, task_repo)
    }

    async fn create_test_tasks(repo: &InMemoryTaskRepository) -> Vec<Task> {
        let task1 = Task::new("Task 1".to_string(), 4).unwrap();
        let task2 = Task::new("Task 2".to_string(), 3).unwrap();
        let task3 = Task::new("Task 3".to_string(), 2).unwrap();

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
        let second_task = service.get_next_task(Some(first_task_id.clone())).await.unwrap().unwrap();
        assert_ne!(second_task.id, first_task_id);

        // After cycling through all tasks, should return to the first task again
        let third_task = service.get_next_task(Some(second_task.id)).await.unwrap().unwrap();
        let fourth_task = service.get_next_task(Some(third_task.id)).await.unwrap().unwrap();
        
        // After cycling through all 3 tasks, should get back to first task
        assert_eq!(fourth_task.id, first_task_id);
    }

    #[tokio::test]
    async fn should_switch_to_specific_task() {
        let (service, repo) = setup_service().await;
        let tasks = create_test_tasks(&repo).await;

        let switched_task = service.switch_to_task(tasks[1].id.clone()).await.unwrap().unwrap();
        assert_eq!(switched_task.name, "Task 2");
        assert_eq!(switched_task.id, tasks[1].id);
    }

    #[tokio::test]
    async fn should_fail_to_switch_to_completed_task() {
        let (service, repo) = setup_service().await;
        let mut task = Task::new("Completed Task".to_string(), 1).unwrap();
        task.increment_session().unwrap(); // Complete the task
        let task_id = task.id.clone();
        repo.create(task).await.unwrap();

        let result = service.switch_to_task(task_id).await;
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

        let next_task = service.cycle_to_next_active_task(Some(tasks[0].id.clone())).await.unwrap().unwrap();
        // Should return a different task than the current one
        assert_ne!(next_task.id, tasks[0].id);
        // Should be one of the available tasks
        assert!(tasks.iter().any(|t| t.id == next_task.id));
    }

    #[tokio::test]
    async fn should_handle_manual_cycling_strategy() {
        let task_repo = Arc::new(InMemoryTaskRepository::new());
        let event_publisher = Arc::new(crate::NoOpEventPublisher);
        let service = DefaultTaskCyclingService::new(
            task_repo.clone(),
            event_publisher,
            TaskCyclingStrategy::Manual,
        );

        let tasks = create_test_tasks(&task_repo).await;

        // In manual mode, should return current task, not cycle
        let next_task = service.get_next_task(Some(tasks[0].id.clone())).await.unwrap().unwrap();
        assert_eq!(next_task.id, tasks[0].id);
        assert_eq!(next_task.name, "Task 1");
    }
}