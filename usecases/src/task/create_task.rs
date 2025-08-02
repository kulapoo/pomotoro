use domain::{Task, TaskBuilder, TaskRepository, EventPublisher, TaskCreated, Result};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CreateTaskCmd {
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
}

pub async fn create_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: CreateTaskCmd,
) -> Result<Task> {
    let mut builder = TaskBuilder::with_name_and_sessions(cmd.name, cmd.max_sessions);
    
    if let Some(description) = cmd.description {
        builder = builder.with_description(description);
    }
    
    if !cmd.tags.is_empty() {
        builder = builder.with_tags(cmd.tags);
    }
    
    let task = builder.build()?;
    
    task_repo.create(task.clone()).await?;
    
    // Publish TaskCreated event
    let created_event = TaskCreated::new(
        task.id.clone(),
        task.name.clone(),
        task.description.clone(),
        task.max_sessions,
        task.tags.clone(),
        task.config.clone(),
        task.audio_config.clone(),
        1, // version
    );
    event_publisher.publish(Box::new(created_event));
    
    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{InMemoryTaskRepository, NoOpEventPublisher};

    #[tokio::test]
    async fn should_create_task_successfully() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        
        let cmd = CreateTaskCmd {
            name: "Test Task".to_string(),
            description: Some("Test description".to_string()),
            max_sessions: 4,
            tags: vec!["work".to_string(), "urgent".to_string()],
        };
        
        let task = create_task(&task_repo, &event_publisher, cmd).await.unwrap();
        
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, Some("Test description".to_string()));
        assert_eq!(task.max_sessions, 4);
        assert_eq!(task.tags, vec!["work".to_string(), "urgent".to_string()]);
        
        // Verify task was saved to repository
        let saved_task = task_repo.get_by_id(task.id).await.unwrap().unwrap();
        assert_eq!(saved_task.name, "Test Task");
    }

    #[tokio::test]
    async fn should_fail_with_empty_name() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        
        let cmd = CreateTaskCmd {
            name: "".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        };
        
        let result = create_task(&task_repo, &event_publisher, cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_fail_with_zero_max_sessions() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        
        let cmd = CreateTaskCmd {
            name: "Test Task".to_string(),
            description: None,
            max_sessions: 0,
            tags: vec![],
        };
        
        let result = create_task(&task_repo, &event_publisher, cmd).await;
        assert!(result.is_err());
    }
}