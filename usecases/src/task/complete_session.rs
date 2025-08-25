use domain::{
    Error, EventPublisher, Result, TaskCompleted, TaskId, TaskRepository,
    TaskSessionCompleted,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SessionCompletionResult {
    pub task_completed: bool,
    pub sessions_completed: u8,
    pub total_sessions: u8,
}

pub async fn complete_session(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    task_id: &str,
) -> Result<SessionCompletionResult> {
    let task_id =
        TaskId::from_string(task_id).map_err(|_| Error::TaskNotFound {
            id: task_id.to_string(),
        })?;

    let mut task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_string(),
        }
    })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    task.increment_session()?;
    let is_task_completed = task.is_completed();

    task_repo.update(task.clone()).await?;

    let session_event = TaskSessionCompleted::new(
        task.id,
        task.current_sessions,
        task.max_sessions,
        is_task_completed,
        task.current_sessions as u64,
    );

    event_publisher.publish(Box::new(session_event));

    if is_task_completed {
        let completed_event = TaskCompleted::new(
            task.id,
            task.current_sessions,
            task.current_sessions as u64 + 1,
        );

        event_publisher.publish(Box::new(completed_event));
    }

    Ok(SessionCompletionResult {
        task_completed: is_task_completed,
        sessions_completed: task.current_sessions,
        total_sessions: task.max_sessions,
    })
}

pub async fn can_complete_session(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    task_id: &str,
) -> Result<bool> {
    let task_id =
        TaskId::from_string(task_id).map_err(|_| Error::TaskNotFound {
            id: task_id.to_string(),
        })?;

    let task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_string(),
        }
    })?;

    Ok(!task.is_completed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::InMemoryTaskRepository;
    use domain::{NoOpEventPublisher, Task, TaskId};

    async fn setup() -> (
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
    ) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(NoOpEventPublisher);
        (task_repo, event_publisher)
    }

    #[tokio::test]
    async fn should_complete_session_successfully() {
        let (task_repo, event_publisher) = setup().await;

        let task = Task::new("Test Task".to_string(), 3).unwrap();
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        let result = complete_session(&task_repo, &event_publisher, &task_id)
            .await
            .unwrap();

        assert!(!result.task_completed);
        assert_eq!(result.sessions_completed, 1);
        assert_eq!(result.total_sessions, 3);

        let updated_task = task_repo
            .get_by_id(TaskId::from_string(&task_id).unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_task.current_sessions, 1);
    }

    #[tokio::test]
    async fn should_complete_task_when_max_sessions_reached() {
        let (task_repo, event_publisher) = setup().await;

        let task = Task::new("Test Task".to_string(), 1).unwrap();
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        let result = complete_session(&task_repo, &event_publisher, &task_id)
            .await
            .unwrap();

        assert!(result.task_completed);
        assert_eq!(result.sessions_completed, 1);
        assert_eq!(result.total_sessions, 1);

        let updated_task = task_repo
            .get_by_id(TaskId::from_string(&task_id).unwrap())
            .await
            .unwrap()
            .unwrap();
        assert!(updated_task.is_completed());
    }

    #[tokio::test]
    async fn should_fail_to_complete_already_completed_task() {
        let (task_repo, event_publisher) = setup().await;

        let mut task = Task::new("Test Task".to_string(), 1).unwrap();
        task.increment_session().unwrap();
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        let result =
            complete_session(&task_repo, &event_publisher, &task_id).await;
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }

    #[tokio::test]
    async fn should_check_if_can_complete_session() {
        let (task_repo, event_publisher) = setup().await;

        let task = Task::new("Test Task".to_string(), 2).unwrap();
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        assert!(can_complete_session(&task_repo, &task_id).await.unwrap());

        complete_session(&task_repo, &event_publisher, &task_id)
            .await
            .unwrap();
        complete_session(&task_repo, &event_publisher, &task_id)
            .await
            .unwrap();

        assert!(!can_complete_session(&task_repo, &task_id).await.unwrap());
    }
}
