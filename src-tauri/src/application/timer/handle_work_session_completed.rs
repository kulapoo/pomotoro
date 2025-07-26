use pomotoro_domain::{WorkSessionCompleted, TaskRepository, EventPublisher, Result};
use crate::application::task::complete_session as complete_task_session;
use std::sync::Arc;

/// Application use case that handles WorkSessionCompleted domain events
/// and orchestrates task session completion through the task domain
pub async fn handle_work_session_completed(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    event: &WorkSessionCompleted,
) -> Result<()> {
    // Only handle if there's an active task
    if let Some(task_id) = &event.active_task_id {
        // Complete the task session using the task domain use case
        complete_task_session(task_repo, event_publisher, &task_id.to_string()).await?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::InMemoryTaskRepository;
    use pomotoro_domain::{Task, TaskDefaults, NoOpEventPublisher, TaskId};

    #[tokio::test]
    async fn should_complete_task_session_when_event_has_task_id() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::empty());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        
        // Create and save a task
        let task = Task::new("Test Task".to_string(), 2).unwrap();
        let task_id = task.id.clone();
        task_repo.create(task).await.unwrap();
        
        // Create work session completed event
        let event = WorkSessionCompleted::new(
            Some(task_id.clone()),
            1500, // 25 minutes
            1,    // session count
            1,    // task session count
            1,    // version
        );
        
        // Handle the event
        let result = handle_work_session_completed(&task_repo, &event_publisher, &event).await;
        assert!(result.is_ok());
        
        // Verify task session was incremented
        let updated_task = task_repo.get_by_id(task_id).await.unwrap().unwrap();
        assert_eq!(updated_task.current_sessions, 1);
    }
    
    #[tokio::test]
    async fn should_do_nothing_when_event_has_no_task_id() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::empty());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        
        // Create work session completed event without task ID
        let event = WorkSessionCompleted::new(
            None,
            1500, // 25 minutes
            1,    // session count
            0,    // task session count
            1,    // version
        );
        
        // Handle the event - should succeed but do nothing
        let result = handle_work_session_completed(&task_repo, &event_publisher, &event).await;
        assert!(result.is_ok());
    }
}