use pomotoro_domain::{TaskRepository, Result, Error, TaskId};
use std::sync::Arc;

pub async fn reset_sessions(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    task_id: &str,
) -> Result<()> {
    let task_id = TaskId::from_string(task_id)
        .map_err(|_| Error::TaskNotFound {
            id: task_id.to_string()
        })?;

    let mut task = task_repo
        .get_by_id(task_id.clone())
        .await?
        .ok_or_else(|| Error::TaskNotFound {
            id: task_id.to_string()
        })?;

    task.reset_sessions();
    task_repo.update(task).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{Task, TaskId, TaskDefaults};
    use crate::infrastructure::InMemoryTaskRepository;

    #[tokio::test]
    async fn should_reset_sessions() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());

        let defaults = TaskDefaults::default();
        let mut task = Task::new("Test Task".to_string(), 3, &defaults).unwrap();
        task.increment_session().unwrap(); // Complete one session
        let task_id = task.id.to_string();
        task_repo.create(task).await.unwrap();

        reset_sessions(&task_repo, &task_id).await.unwrap();

        let updated_task = task_repo.get_by_id(TaskId::from_string(&task_id).unwrap())
            .await.unwrap().unwrap();
        assert_eq!(updated_task.current_sessions, 0);
        assert!(!updated_task.is_completed());
    }

    #[tokio::test]
    async fn should_fail_for_nonexistent_task() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());

        let result = reset_sessions(&task_repo, "nonexistent").await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }
}