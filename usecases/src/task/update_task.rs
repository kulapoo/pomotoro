use domain::{Task, TaskId, TaskRepository, EventPublisher, TaskConfig, AudioConfig, TaskUpdated, Result, Error};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct UpdateTaskCmd {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub config: Option<TaskConfig>,
    pub audio_config: Option<AudioConfig>,
}

pub async fn update_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: UpdateTaskCmd,
) -> Result<Task> {
    let task_id = TaskId::from_string(&cmd.id)
        .map_err(|_| Error::TaskNotFound { id: cmd.id.clone() })?;
    
    let mut task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or_else(|| Error::TaskNotFound { id: cmd.id.clone() })?;
    
    // Prevent updating completed tasks
    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }
    
    // Capture update info for event before modifying task
    let updated_name = cmd.name.clone();
    let updated_description = cmd.description.clone();
    let updated_max_sessions = cmd.max_sessions;
    let updated_tags = cmd.tags.clone();
    
    // Update task fields if provided
    if let Some(name) = cmd.name {
        if name.trim().is_empty() {
            return Err(Error::InvalidSessionCount { count: 0 });
        }
        task.name = name.trim().to_string();
    }
    
    if let Some(description) = cmd.description {
        task.description = Some(description);
    }
    
    if let Some(max_sessions) = cmd.max_sessions {
        if max_sessions == 0 {
            return Err(Error::InvalidSessionCount { count: max_sessions });
        }
        task.max_sessions = max_sessions;
        
        // If current sessions exceed new max, mark as completed
        if task.current_sessions >= max_sessions {
            task.status = domain::TaskStatus::Completed;
            task.completed_at = Some(chrono::Utc::now());
        }
    }
    
    if let Some(tags) = cmd.tags {
        task.tags = tags;
    }
    
    if let Some(config) = cmd.config {
        // TaskConfig is already validated at construction, no need to validate again
        task.config = config;
    }
    
    if let Some(audio_config) = cmd.audio_config {
        audio_config.validate()?;
        task.audio_config = audio_config;
    }
    
    task_repo.update(task.clone()).await?;
    
    // Publish TaskUpdated event  
    let updated_event = TaskUpdated::new(
        task.id.clone(),
        updated_name,
        updated_description,
        updated_max_sessions,
        updated_tags,
        1, // version
    );
    event_publisher.publish(Box::new(updated_event));
    
    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::NoOpEventPublisher;
    use domain::InMemoryTaskRepository;

    async fn setup() -> (Arc<dyn TaskRepository + Send + Sync>, Arc<dyn EventPublisher + Send + Sync>, Task) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        
        let task = Task::new("Original Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        (task_repo, event_publisher, task)
    }

    #[tokio::test]
    async fn should_update_task_name() {
        let (task_repo, event_publisher, original_task) = setup().await;
        
        let cmd = UpdateTaskCmd {
            id: original_task.id.to_string(),
            name: Some("Updated Task".to_string()),
            description: None,
            max_sessions: None,
            tags: None,
            config: None,
            audio_config: None,
        };
        
        let updated_task = update_task(&task_repo, &event_publisher, cmd).await.unwrap();
        
        assert_eq!(updated_task.name, "Updated Task");
        assert_eq!(updated_task.id, original_task.id);
        assert_eq!(updated_task.max_sessions, original_task.max_sessions);
    }

    #[tokio::test]
    async fn should_update_multiple_fields() {
        let (task_repo, event_publisher, original_task) = setup().await;
        
        let cmd = UpdateTaskCmd {
            id: original_task.id.to_string(),
            name: Some("New Name".to_string()),
            description: Some("New description".to_string()),
            max_sessions: Some(6),
            tags: Some(vec!["updated".to_string(), "test".to_string()]),
            config: None,
            audio_config: None,
        };
        
        let updated_task = update_task(&task_repo, &event_publisher, cmd).await.unwrap();
        
        assert_eq!(updated_task.name, "New Name");
        assert_eq!(updated_task.description, Some("New description".to_string()));
        assert_eq!(updated_task.max_sessions, 6);
        assert_eq!(updated_task.tags, vec!["updated".to_string(), "test".to_string()]);
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (task_repo, event_publisher, _) = setup().await;
        
        let cmd = UpdateTaskCmd {
            id: "nonexistent-id".to_string(),
            name: Some("New Name".to_string()),
            description: None,
            max_sessions: None,
            tags: None,
            config: None,
            audio_config: None,
        };
        
        let result = update_task(&task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn should_fail_to_update_completed_task() {
        let (task_repo, event_publisher, _) = setup().await;
        
        let mut completed_task = Task::new("Completed Task".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap();
        task_repo.create(completed_task.clone()).await.unwrap();
        
        let cmd = UpdateTaskCmd {
            id: completed_task.id.to_string(),
            name: Some("Should not update".to_string()),
            description: None,
            max_sessions: None,
            tags: None,
            config: None,
            audio_config: None,
        };
        
        let result = update_task(&task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }

    #[tokio::test]
    async fn should_complete_task_when_current_sessions_exceed_new_max() {
        let (task_repo, event_publisher, original_task) = setup().await;
        
        // Complete 2 sessions first
        let mut task_with_sessions = original_task.clone();
        task_with_sessions.increment_session().unwrap();
        task_with_sessions.increment_session().unwrap();
        task_repo.update(task_with_sessions).await.unwrap();
        
        let cmd = UpdateTaskCmd {
            id: original_task.id.to_string(),
            name: None,
            description: None,
            max_sessions: Some(1), // Less than current sessions (2)
            tags: None,
            config: None,
            audio_config: None,
        };
        
        let updated_task = update_task(&task_repo, &event_publisher, cmd).await.unwrap();
        
        assert_eq!(updated_task.max_sessions, 1);
        assert!(updated_task.is_completed());
        assert!(updated_task.completed_at.is_some());
    }
}