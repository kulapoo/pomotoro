use pomotoro_domain::{
    Error, Result, Task, TaskCyclerService, TaskId, TaskRepository, TaskStatus,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskQueueQuery {
    pub include_completed: bool,
    pub active_task_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskQueueInfo {
    pub tasks: Vec<Task>,
    pub active_task_id: Option<TaskId>,
    pub current_position: Option<usize>,
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
}

pub async fn get_task_queue(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    query: TaskQueueQuery,
) -> Result<TaskQueueInfo> {
    let active_task_id = if let Some(id_str) = query.active_task_id {
        Some(TaskId::from_string(&id_str).map_err(|_| Error::TaskNotFound { id: id_str })?)
    } else {
        None
    };

    // Get tasks based on query parameters
    let tasks = if query.include_completed {
        task_repo.get_all().await?
    } else {
        cycling_service.get_active_task_queue().await?
    };

    // Calculate statistics
    let total_tasks = tasks.len();
    let active_tasks = tasks.iter().filter(|t| !t.is_completed()).count();
    let completed_tasks = tasks.iter().filter(|t| t.is_completed()).count();

    // Find current position if active task is specified
    let current_position = if let Some(active_id) = &active_task_id {
        tasks.iter().position(|t| t.id == *active_id)
    } else {
        None
    };

    Ok(TaskQueueInfo {
        tasks,
        active_task_id,
        current_position,
        total_tasks,
        active_tasks,
        completed_tasks,
    })
}

pub async fn get_active_task_queue(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
) -> Result<Vec<Task>> {
    cycling_service.get_active_task_queue().await
}

pub async fn get_task_queue_with_priorities(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    active_task_id: Option<String>,
) -> Result<TaskQueueInfo> {
    let active_id = if let Some(id_str) = active_task_id {
        Some(TaskId::from_string(&id_str).map_err(|_| Error::TaskNotFound { id: id_str })?)
    } else {
        None
    };

    // Get active tasks and sort them by priority logic
    let mut tasks = cycling_service.get_active_task_queue().await?;

    // Sort by: 1) Active task first, 2) Incomplete tasks by creation date, 3) Completed tasks last
    tasks.sort_by(|a, b| {
        // Active task goes first
        if let Some(active_id) = &active_id {
            if a.id == *active_id {
                return std::cmp::Ordering::Less;
            }
            if b.id == *active_id {
                return std::cmp::Ordering::Greater;
            }
        }

        // Then by completion status (incomplete first)
        match (a.is_completed(), b.is_completed()) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => {
                // Finally by creation date (newer first for incomplete, older first for completed)
                if !a.is_completed() && !b.is_completed() {
                    b.created_at.cmp(&a.created_at)
                } else {
                    a.created_at.cmp(&b.created_at)
                }
            }
        }
    });

    let total_tasks = tasks.len();
    let active_tasks = tasks.iter().filter(|t| !t.is_completed()).count();
    let completed_tasks = tasks.iter().filter(|t| t.is_completed()).count();

    let current_position = if let Some(active_id) = &active_id {
        tasks.iter().position(|t| t.id == *active_id)
    } else {
        None
    };

    Ok(TaskQueueInfo {
        tasks,
        active_task_id: active_id,
        current_position,
        total_tasks,
        active_tasks,
        completed_tasks,
    })
}

pub async fn get_task_queue_summary(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
) -> Result<TaskQueueSummary> {
    let all_tasks = task_repo.get_all().await?;
    let active_tasks = cycling_service.get_active_task_queue().await?;

    let total_tasks = all_tasks.len();
    let active_count = active_tasks.len();
    let completed_count = all_tasks.iter().filter(|t| t.is_completed()).count();
    let paused_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Paused)
        .count();

    // Calculate total sessions and completed sessions
    let total_sessions: u32 = all_tasks.iter().map(|t| t.max_sessions as u32).sum();
    let completed_sessions: u32 = all_tasks.iter().map(|t| t.current_sessions as u32).sum();

    Ok(TaskQueueSummary {
        total_tasks,
        active_tasks: active_count,
        completed_tasks: completed_count,
        paused_tasks: paused_count,
        total_sessions,
        completed_sessions,
        progress_percentage: if total_sessions > 0 {
            (completed_sessions as f64 / total_sessions as f64) * 100.0
        } else {
            0.0
        },
    })
}

#[derive(Debug, Clone)]
pub struct TaskQueueSummary {
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub paused_tasks: usize,
    pub total_sessions: u32,
    pub completed_sessions: u32,
    pub progress_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::InMemoryTaskRepository;
    use pomotoro_domain::{
        EventPublisher, NoOpEventPublisher, TaskCyclingStrategy,
    };
    use crate::infrastructure::StandardTaskCyclerService;

    async fn setup() -> (
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
        Arc<dyn TaskCyclerService + Send + Sync>,
        Vec<Task>,
    ) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::empty());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        let cycling_service: Arc<dyn TaskCyclerService + Send + Sync> = Arc::new(
            StandardTaskCyclerService::new(task_repo.clone(), TaskCyclingStrategy::RoundRobin),
        );

        let task1 = Task::new("Task 1".to_string(), 4).unwrap();
        let task2 = Task::new("Task 2".to_string(), 3).unwrap();
        let mut task3 = Task::new("Task 3".to_string(), 2).unwrap();
        task3.increment_session().unwrap();
        task3.increment_session().unwrap(); // Complete task3

        task_repo.create(task1.clone()).await.unwrap();
        task_repo.create(task2.clone()).await.unwrap();
        task_repo.create(task3.clone()).await.unwrap();

        (
            task_repo,
            event_publisher,
            cycling_service,
            vec![task1, task2, task3],
        )
    }

    #[tokio::test]
    async fn should_get_task_queue_without_completed() {
        let (task_repo, _, cycling_service, tasks) = setup().await;

        let query = TaskQueueQuery {
            include_completed: false,
            active_task_id: Some(tasks[0].id.to_string()),
        };

        let queue_info = get_task_queue(&task_repo, &cycling_service, query)
            .await
            .unwrap();

        assert_eq!(queue_info.tasks.len(), 2); // Only active tasks
        assert_eq!(queue_info.active_tasks, 2);
        assert_eq!(queue_info.completed_tasks, 0); // Not included in results
        assert_eq!(queue_info.current_position, Some(0));
        assert_eq!(queue_info.active_task_id, Some(tasks[0].id.clone()));
    }

    #[tokio::test]
    async fn should_get_task_queue_with_completed() {
        let (task_repo, _, cycling_service, tasks) = setup().await;

        let query = TaskQueueQuery {
            include_completed: true,
            active_task_id: Some(tasks[1].id.to_string()),
        };

        let queue_info = get_task_queue(&task_repo, &cycling_service, query)
            .await
            .unwrap();

        assert_eq!(queue_info.tasks.len(), 3); // All tasks
        assert_eq!(queue_info.total_tasks, 3);
        assert_eq!(queue_info.active_tasks, 2);
        assert_eq!(queue_info.completed_tasks, 1);
        assert!(queue_info.current_position.is_some());
    }

    #[tokio::test]
    async fn should_get_active_task_queue_only() {
        let (_task_repo, _, cycling_service, _) = setup().await;

        let active_tasks = get_active_task_queue(&cycling_service).await.unwrap();

        assert_eq!(active_tasks.len(), 2); // Only active tasks
        assert!(active_tasks.iter().all(|t| !t.is_completed()));
    }

    #[tokio::test]
    async fn should_get_task_queue_with_priorities() {
        let (_task_repo, _, cycling_service, tasks) = setup().await;

        let queue_info =
            get_task_queue_with_priorities(&cycling_service, Some(tasks[1].id.to_string()))
                .await
                .unwrap();

        // Active task should be first
        assert_eq!(queue_info.tasks[0].id, tasks[1].id);
        assert_eq!(queue_info.current_position, Some(0));
    }

    #[tokio::test]
    async fn should_get_task_queue_summary() {
        let (task_repo, _, cycling_service, _) = setup().await;

        let summary = get_task_queue_summary(&task_repo, &cycling_service)
            .await
            .unwrap();

        assert_eq!(summary.total_tasks, 3);
        assert_eq!(summary.active_tasks, 2);
        assert_eq!(summary.completed_tasks, 1);
        assert_eq!(summary.total_sessions, 9); // 4 + 3 + 2
        assert_eq!(summary.completed_sessions, 2); // Only task3 completed 2 sessions
        assert!((summary.progress_percentage - 22.22).abs() < 0.01); // ~22.22%
    }

    #[tokio::test]
    async fn should_handle_empty_queue() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::empty());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        let cycling_service: Arc<dyn TaskCyclerService + Send + Sync> = Arc::new(
            StandardTaskCyclerService::new(task_repo.clone(), TaskCyclingStrategy::RoundRobin),
        );

        let query = TaskQueueQuery {
            include_completed: false,
            active_task_id: None,
        };

        let queue_info = get_task_queue(&task_repo, &cycling_service, query)
            .await
            .unwrap();

        assert_eq!(queue_info.tasks.len(), 0);
        assert_eq!(queue_info.total_tasks, 0);
        assert_eq!(queue_info.active_tasks, 0);
        assert_eq!(queue_info.completed_tasks, 0);
        assert_eq!(queue_info.current_position, None);
    }

    #[tokio::test]
    async fn should_fail_with_invalid_active_task_id() {
        let (task_repo, _, cycling_service, _) = setup().await;

        let query = TaskQueueQuery {
            include_completed: false,
            active_task_id: Some("invalid-task-id".to_string()),
        };

        let result = get_task_queue(&task_repo, &cycling_service, query).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }
}
