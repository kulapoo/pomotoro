use domain::{Task, TaskId, TaskRepository, EventPublisher, TaskUpdated, Result, Error};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SetDefaultTaskCmd {
    pub task_id: String,
}

pub async fn set_default_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: SetDefaultTaskCmd,
) -> Result<Task> {
    let task_id = TaskId::from_string(&cmd.task_id)
        .map_err(|_| Error::TaskNotFound { id: cmd.task_id.clone() })?;
    
    let mut task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or_else(|| Error::TaskNotFound { id: cmd.task_id.clone() })?;
    
    if task.is_default() {
        return Ok(task);
    }
    
    if let Some(mut current_default) = task_repo.get_default_task().await? {
        current_default.unset_as_default();
        task_repo.update(current_default.clone()).await?;
        
        // Publish event for the previously default task
        let updated_event = TaskUpdated::new(
            current_default.id,
            None,
            None,
            None,
            None,
            1
        );
        event_publisher.publish(Box::new(updated_event));
    }
    
    task.set_as_default();
    task_repo.update(task.clone()).await?;
    
    // Publish event for the newly default task
    let updated_event = TaskUpdated::new(
        task.id,
        None,
        None,
        None,
        None,
        1
    );
    event_publisher.publish(Box::new(updated_event));
    
    Ok(task)
}

pub async fn get_default_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
) -> Result<Option<Task>> {
    task_repo.get_default_task().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{InMemoryTaskRepository, NoOpEventPublisher, Task};

    async fn setup() -> (Arc<dyn TaskRepository + Send + Sync>, Arc<dyn EventPublisher + Send + Sync>) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        (task_repo, event_publisher)
    }

    #[tokio::test]
    async fn should_set_default_task() {
        let (task_repo, event_publisher) = setup().await;
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        let cmd = SetDefaultTaskCmd {
            task_id: task.id.to_string(),
        };
        
        let result = set_default_task(&task_repo, &event_publisher, cmd).await.unwrap();
        
        assert!(result.is_default());
        
        // Verify in repository
        let default_task = task_repo.get_default_task().await.unwrap();
        assert!(default_task.is_some());
        assert_eq!(default_task.unwrap().id, task.id);
    }

    #[tokio::test]
    async fn should_unset_previous_default_when_setting_new_one() {
        let (task_repo, event_publisher) = setup().await;
        
        // Create first task and set as default
        let task1 = Task::new("Task 1".to_string(), 4).unwrap();
        task_repo.create(task1.clone()).await.unwrap();
        let cmd1 = SetDefaultTaskCmd {
            task_id: task1.id.to_string(),
        };
        set_default_task(&task_repo, &event_publisher, cmd1).await.unwrap();
        
        // Create second task and set as default
        let task2 = Task::new("Task 2".to_string(), 4).unwrap();
        task_repo.create(task2.clone()).await.unwrap();
        let cmd2 = SetDefaultTaskCmd {
            task_id: task2.id.to_string(),
        };
        let result = set_default_task(&task_repo, &event_publisher, cmd2).await.unwrap();
        
        assert!(result.is_default());
        assert_eq!(result.id, task2.id);
        
        // Verify only one default task exists
        let default_task = task_repo.get_default_task().await.unwrap();
        assert!(default_task.is_some());
        assert_eq!(default_task.unwrap().id, task2.id);
        
        // Verify first task is no longer default
        let first_task = task_repo.get_by_id(task1.id).await.unwrap().unwrap();
        assert!(!first_task.is_default());
    }

    #[tokio::test]
    async fn should_return_task_if_already_default() {
        let (task_repo, event_publisher) = setup().await;
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        let cmd = SetDefaultTaskCmd {
            task_id: task.id.to_string(),
        };
        
        set_default_task(&task_repo, &event_publisher, cmd.clone()).await.unwrap();
        
        // Should be idempotent
        let result = set_default_task(&task_repo, &event_publisher, cmd).await.unwrap();
        
        assert!(result.is_default());
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (task_repo, event_publisher) = setup().await;
        
        let cmd = SetDefaultTaskCmd {
            task_id: "nonexistent-id".to_string(),
        };
        
        let result = set_default_task(&task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn should_get_default_task() {
        let (task_repo, event_publisher) = setup().await;
        
        // Initially no default task
        let default_task = get_default_task(&task_repo).await.unwrap();
        assert!(default_task.is_none());
        
        // Create and set default task
        let task = Task::new("Default Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        let cmd = SetDefaultTaskCmd {
            task_id: task.id.to_string(),
        };
        set_default_task(&task_repo, &event_publisher, cmd).await.unwrap();
        
        // Verify we can get the default task
        let default_task = get_default_task(&task_repo).await.unwrap();
        assert!(default_task.is_some());
        assert_eq!(default_task.unwrap().id, task.id);
    }
}