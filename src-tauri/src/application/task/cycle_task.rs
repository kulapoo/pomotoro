use pomotoro_domain::{
    Task, TaskId, TaskRepository, TaskCyclingService,
    Result, Error
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GetNextTaskQuery {
    pub current_task_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskCycleResult {
    pub next_task: Option<Task>,
    pub has_more_tasks: bool,
    pub cycle_position: usize,
    pub total_tasks: usize,
}

pub async fn get_next_task(
    _task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclingService + Send + Sync>,
    query: GetNextTaskQuery,
) -> Result<Option<Task>> {
    let current_task_id = if let Some(id_str) = query.current_task_id {
        Some(TaskId::from_string(&id_str)
            .map_err(|_| Error::TaskNotFound { id: id_str })?)
    } else {
        None
    };
    
    cycling_service.get_next_task(current_task_id).await
}

pub async fn cycle_to_next_task(
    _task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclingService + Send + Sync>,
    current_task_id: Option<String>,
) -> Result<TaskCycleResult> {
    let current_id = if let Some(id_str) = current_task_id {
        Some(TaskId::from_string(&id_str)
            .map_err(|_| Error::TaskNotFound { id: id_str })?)
    } else {
        None
    };
    
    let next_task = cycling_service
        .cycle_to_next_active_task(current_id.clone())
        .await?;
    
    // Get task queue information for context
    let active_tasks = cycling_service.get_active_task_queue().await?;
    let total_tasks = active_tasks.len();
    
    let cycle_position = if let (Some(next), Some(_current)) = (&next_task, &current_id) {
        active_tasks
            .iter()
            .position(|t| t.id == next.id)
            .unwrap_or(0)
    } else {
        0
    };
    
    let has_more_tasks = total_tasks > 1;
    
    Ok(TaskCycleResult {
        next_task,
        has_more_tasks,
        cycle_position,
        total_tasks,
    })
}

pub async fn get_task_cycle_info(
    _task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclingService + Send + Sync>,
    current_task_id: Option<String>,
) -> Result<TaskCycleResult> {
    let current_id = if let Some(id_str) = current_task_id {
        Some(TaskId::from_string(&id_str)
            .map_err(|_| Error::TaskNotFound { id: id_str })?)
    } else {
        None
    };
    
    let active_tasks = cycling_service.get_active_task_queue().await?;
    let total_tasks = active_tasks.len();
    
    let current_task = if let Some(current_id) = current_id {
        active_tasks.iter().find(|t| t.id == current_id).cloned()
    } else {
        None
    };
    
    let cycle_position = if let Some(current) = &current_task {
        active_tasks
            .iter()
            .position(|t| t.id == current.id)
            .unwrap_or(0)
    } else {
        0
    };
    
    let has_more_tasks = total_tasks > 1;
    
    Ok(TaskCycleResult {
        next_task: current_task,
        has_more_tasks,
        cycle_position,
        total_tasks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{
        NoOpEventPublisher, EventPublisher,
        DefaultTaskCyclingService, TaskCyclingStrategy, TaskDefaults
    };
    use crate::infrastructure::InMemoryTaskRepository;

    async fn setup() -> (
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
        Arc<dyn TaskCyclingService + Send + Sync>,
        Vec<Task>,
    ) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::empty());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        let cycling_service: Arc<dyn TaskCyclingService + Send + Sync> = Arc::new(DefaultTaskCyclingService::new(
            task_repo.clone(),
            TaskCyclingStrategy::RoundRobin,
        ));
        
        let defaults = TaskDefaults::default();
        let task1 = Task::new("Task 1".to_string(), 4, &defaults).unwrap();
        let task2 = Task::new("Task 2".to_string(), 3, &defaults).unwrap();
        let task3 = Task::new("Task 3".to_string(), 2, &defaults).unwrap();
        
        task_repo.create(task1.clone()).await.unwrap();
        task_repo.create(task2.clone()).await.unwrap();
        task_repo.create(task3.clone()).await.unwrap();
        
        (task_repo, event_publisher, cycling_service, vec![task1, task2, task3])
    }

    #[tokio::test]
    async fn should_get_next_task_with_current_id() {
        let (task_repo, _, cycling_service, tasks) = setup().await;
        
        let query = GetNextTaskQuery {
            current_task_id: Some(tasks[0].id.to_string()),
        };
        
        let next_task = get_next_task(&task_repo, &cycling_service, query)
            .await
            .unwrap()
            .unwrap();
        
        // Should get a different task from the current one
        assert_ne!(next_task.id, tasks[0].id);
        assert!(tasks.iter().any(|t| t.id == next_task.id));
    }

    #[tokio::test]
    async fn should_get_first_task_without_current_id() {
        let (task_repo, _, cycling_service, tasks) = setup().await;
        
        let query = GetNextTaskQuery {
            current_task_id: None,
        };
        
        let next_task = get_next_task(&task_repo, &cycling_service, query)
            .await
            .unwrap()
            .unwrap();
        
        // Should get some task from available tasks
        assert!(tasks.iter().any(|t| t.id == next_task.id));
    }

    #[tokio::test]
    async fn should_cycle_to_next_task() {
        let (task_repo, _, cycling_service, tasks) = setup().await;
        
        let result = cycle_to_next_task(
            &task_repo,
            &cycling_service,
            Some(tasks[0].id.to_string()),
        ).await.unwrap();
        
        assert!(result.next_task.is_some());
        assert!(result.has_more_tasks);
        assert_eq!(result.total_tasks, 3);
        
        let next_task = result.next_task.unwrap();
        assert_ne!(next_task.id, tasks[0].id);
        assert!(tasks.iter().any(|t| t.id == next_task.id));
    }

    #[tokio::test]
    async fn should_get_task_cycle_info() {
        let (task_repo, _, cycling_service, tasks) = setup().await;
        
        let result = get_task_cycle_info(
            &task_repo,
            &cycling_service,
            Some(tasks[1].id.to_string()),
        ).await.unwrap();
        
        assert!(result.next_task.is_some());
        assert!(result.has_more_tasks);
        assert_eq!(result.total_tasks, 3);
        
        let current_task = result.next_task.unwrap();
        assert_eq!(current_task.id, tasks[1].id);
    }

    #[tokio::test]
    async fn should_handle_single_task() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::empty());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        let cycling_service: Arc<dyn TaskCyclingService + Send + Sync> = Arc::new(DefaultTaskCyclingService::new(
            task_repo.clone(),
            TaskCyclingStrategy::RoundRobin,
        ));
        
        let defaults = TaskDefaults::default();
        let single_task = Task::new("Single Task".to_string(), 4, &defaults).unwrap();
        task_repo.create(single_task.clone()).await.unwrap();
        
        let result = cycle_to_next_task(
            &task_repo,
            &cycling_service,
            Some(single_task.id.to_string()),
        ).await.unwrap();
        
        assert!(result.next_task.is_some());
        assert!(!result.has_more_tasks); // Only one task
        assert_eq!(result.total_tasks, 1);
        assert_eq!(result.cycle_position, 0);
        
        // Should return the same task in round-robin with single task
        let next_task = result.next_task.unwrap();
        assert_eq!(next_task.id, single_task.id);
    }

    #[tokio::test]
    async fn should_handle_no_tasks() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::empty());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        let cycling_service: Arc<dyn TaskCyclingService + Send + Sync> = Arc::new(DefaultTaskCyclingService::new(
            task_repo.clone(),
            TaskCyclingStrategy::RoundRobin,
        ));
        
        let result = cycle_to_next_task(
            &task_repo,
            &cycling_service,
            None,
        ).await.unwrap();
        
        assert!(result.next_task.is_none());
        assert!(!result.has_more_tasks);
        assert_eq!(result.total_tasks, 0);
        assert_eq!(result.cycle_position, 0);
    }

    #[tokio::test]
    async fn should_exclude_completed_tasks_from_cycling() {
        let (task_repo, _, cycling_service, tasks) = setup().await;
        
        // Complete one task
        let mut completed_task = tasks[1].clone();
        completed_task.increment_session().unwrap();
        completed_task.increment_session().unwrap();
        completed_task.increment_session().unwrap(); // Complete all sessions
        task_repo.update(completed_task).await.unwrap();
        
        let result = cycle_to_next_task(
            &task_repo,
            &cycling_service,
            Some(tasks[0].id.to_string()),
        ).await.unwrap();
        
        assert!(result.next_task.is_some());
        assert_eq!(result.total_tasks, 2); // Should exclude completed task
        
        let next_task = result.next_task.unwrap();
        assert_ne!(next_task.id, tasks[1].id); // Should not be the completed task
    }

    #[tokio::test]
    async fn should_fail_with_invalid_task_id() {
        let (task_repo, _, cycling_service, _) = setup().await;
        
        let result = cycle_to_next_task(
            &task_repo,
            &cycling_service,
            Some("invalid-task-id".to_string()),
        ).await;
        
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }
}