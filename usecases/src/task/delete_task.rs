use domain::{Error, EventPublisher, Result, TaskId, TaskRepository};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DeleteTaskCmd {
    pub id: String,
}

pub async fn delete_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: DeleteTaskCmd,
) -> Result<bool> {
    let task_id = TaskId::from_string(&cmd.id)
        .map_err(|_| Error::TaskNotFound { id: cmd.id.clone() })?;

    let task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or_else(|| Error::TaskNotFound { id: cmd.id.clone() })?;

    if task.name == "Focus Session"
        && task.description
            == Some("Default pomodoro task for focused work".to_string())
    {
        return Err(Error::InvalidStateTransition {
            from: "default_task".to_string(),
            to: "deleted".to_string(),
        });
    }

    let deleted = task_repo.delete(task_id).await?;

    if deleted {
        // TODO: Publish TaskDeleted event - domain event not yet implemented
        // let deleted_event = TaskDeleted::new(task.id, task.name);
        // event_publisher.publish(Box::new(deleted_event));
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::InMemoryTaskRepository;
    use domain::{NoOpEventPublisher, Task};

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
    async fn should_delete_task_successfully() {
        let (task_repo, event_publisher) = setup().await;

        let task = Task::new("Test Task".to_string(), 4).unwrap();
        let task_id = task.id;
        task_repo.create(task).await.unwrap();

        let cmd = DeleteTaskCmd {
            id: task_id.to_string(),
        };

        let deleted = delete_task(&task_repo, &event_publisher, cmd)
            .await
            .unwrap();

        assert!(deleted);

        let result = task_repo.get_by_id(task_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (task_repo, event_publisher) = setup().await;

        let cmd = DeleteTaskCmd {
            id: "nonexistent-id".to_string(),
        };

        let result = delete_task(&task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn should_prevent_deletion_of_default_task() {
        let (task_repo, event_publisher) = setup().await;

        let default_task = Task::new_default().unwrap();
        let task_id = default_task.id;
        task_repo.create(default_task).await.unwrap();

        let cmd = DeleteTaskCmd {
            id: task_id.to_string(),
        };

        let result = delete_task(&task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));

        let task = task_repo.get_by_id(task_id).await.unwrap();
        assert!(task.is_some());
    }

    #[tokio::test]
    async fn should_handle_invalid_task_id() {
        let (task_repo, event_publisher) = setup().await;

        let cmd = DeleteTaskCmd {
            id: "invalid-id-format".to_string(),
        };

        let result = delete_task(&task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }
}
