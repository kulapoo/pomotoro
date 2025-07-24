use crate::{TaskRepository, EventPublisher, TaskSessionCompleted, TaskCompleted, Result, Error, TaskId};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait TaskSessionServiceInterface: Send + Sync {
    async fn complete_session(&self, task_id: &str) -> Result<SessionCompletionResult>;
    async fn reset_sessions(&self, task_id: &str) -> Result<()>;
    async fn can_complete_session(&self, task_id: &str) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct SessionCompletionResult {
    pub task_completed: bool,
    pub sessions_completed: u8,
    pub total_sessions: u8,
}

pub struct TaskSessionService {
    task_repository: Arc<dyn TaskRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl TaskSessionService {
    pub fn new(
        task_repository: Arc<dyn TaskRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            task_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl TaskSessionServiceInterface for TaskSessionService {
    async fn complete_session(&self, task_id_str: &str) -> Result<SessionCompletionResult> {
        let task_id = TaskId::from_string(task_id_str)
            .map_err(|_| Error::TaskNotFound {
                id: task_id_str.to_string()
            })?;

        let mut task = self.task_repository
            .get_by_id(task_id.clone())
            .await?
            .ok_or_else(|| Error::TaskNotFound {
                id: task_id_str.to_string()
            })?;

        if task.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }

        let _old_sessions = task.current_sessions;
        task.increment_session()?;
        let is_task_completed = task.is_completed();

        self.task_repository.update(task.clone()).await?;

        // Publish session completed event
        let session_event = TaskSessionCompleted::new(
            task.id.clone(),
            task.current_sessions,
            task.max_sessions,
            is_task_completed,
            task.current_sessions as u64,
        );

        self.event_publisher.publish(Box::new(session_event));

        // If task is completed, publish task completed event
        if is_task_completed {
            let completed_event = TaskCompleted::new(
                task.id.clone(),
                task.current_sessions,
                task.current_sessions as u64 + 1,
            );

            self.event_publisher.publish(Box::new(completed_event));
        }

        Ok(SessionCompletionResult {
            task_completed: is_task_completed,
            sessions_completed: task.current_sessions,
            total_sessions: task.max_sessions,
        })
    }

    async fn reset_sessions(&self, task_id_str: &str) -> Result<()> {
        let task_id = TaskId::from_string(task_id_str)
            .map_err(|_| Error::TaskNotFound {
                id: task_id_str.to_string()
            })?;

        let mut task = self.task_repository
            .get_by_id(task_id.clone())
            .await?
            .ok_or_else(|| Error::TaskNotFound {
                id: task_id_str.to_string()
            })?;

        task.reset_sessions();
        self.task_repository.update(task).await
    }

    async fn can_complete_session(&self, task_id_str: &str) -> Result<bool> {
        let task_id = TaskId::from_string(task_id_str)
            .map_err(|_| Error::TaskNotFound {
                id: task_id_str.to_string()
            })?;

        let task = self.task_repository
            .get_by_id(task_id.clone())
            .await?
            .ok_or_else(|| Error::TaskNotFound {
                id: task_id_str.to_string()
            })?;

        Ok(!task.is_completed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::repo::InMemoryTaskRepository;
    use crate::{Task, TaskId};

    async fn setup_service() -> (TaskSessionService, Arc<InMemoryTaskRepository>) {
        let task_repo = Arc::new(InMemoryTaskRepository::new());
        let event_publisher = Arc::new(crate::NoOpEventPublisher);
        let service = TaskSessionService::new(task_repo.clone(), event_publisher);
        (service, task_repo)
    }

    #[tokio::test]
    async fn should_complete_session_successfully() {
        let (service, task_repo) = setup_service().await;

        let task = Task::new("Test Task".to_string(), 3).unwrap();
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        let result = service.complete_session(&task_id).await.unwrap();

        assert!(!result.task_completed);
        assert_eq!(result.sessions_completed, 1);
        assert_eq!(result.total_sessions, 3);

        let updated_task = task_repo.get_by_id(TaskId::from_string(&task_id).unwrap())
            .await.unwrap().unwrap();
        assert_eq!(updated_task.current_sessions, 1);
    }

    #[tokio::test]
    async fn should_complete_task_when_max_sessions_reached() {
        let (service, task_repo) = setup_service().await;

        let task = Task::new("Test Task".to_string(), 1).unwrap();
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        let result = service.complete_session(&task_id).await.unwrap();

        assert!(result.task_completed);
        assert_eq!(result.sessions_completed, 1);
        assert_eq!(result.total_sessions, 1);

        let updated_task = task_repo.get_by_id(TaskId::from_string(&task_id).unwrap())
            .await.unwrap().unwrap();
        assert!(updated_task.is_completed());
    }

    #[tokio::test]
    async fn should_fail_to_complete_already_completed_task() {
        let (service, task_repo) = setup_service().await;

        let mut task = Task::new("Test Task".to_string(), 1).unwrap();
        task.increment_session().unwrap(); // Complete the task
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        let result = service.complete_session(&task_id).await;
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }

    #[tokio::test]
    async fn should_reset_sessions() {
        let (service, task_repo) = setup_service().await;

        let mut task = Task::new("Test Task".to_string(), 3).unwrap();
        task.increment_session().unwrap(); // Complete one session
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        service.reset_sessions(&task_id).await.unwrap();

        let updated_task = task_repo.get_by_id(TaskId::from_string(&task_id).unwrap())
            .await.unwrap().unwrap();
        assert_eq!(updated_task.current_sessions, 0);
        assert!(!updated_task.is_completed());
    }

    #[tokio::test]
    async fn should_check_if_can_complete_session() {
        let (service, task_repo) = setup_service().await;

        let task = Task::new("Test Task".to_string(), 2).unwrap();
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        assert!(service.can_complete_session(&task_id).await.unwrap());

        // Complete both sessions
        service.complete_session(&task_id).await.unwrap();
        service.complete_session(&task_id).await.unwrap();

        assert!(!service.can_complete_session(&task_id).await.unwrap());
    }
}