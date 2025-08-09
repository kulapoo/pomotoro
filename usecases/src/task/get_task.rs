use domain::{Task, TaskId, TaskRepository, TaskStatus, Result, Error};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GetTaskQuery {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetTasksQuery {
    pub tags: Option<Vec<String>>,
    pub status: Option<TaskStatus>,
    pub active_only: bool,
}

pub async fn get_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    query: GetTaskQuery,
) -> Result<Task> {
    let task_id = TaskId::from_string(&query.id)
        .map_err(|_| Error::TaskNotFound { id: query.id.clone() })?;
    
    task_repo
        .get_by_id(task_id)
        .await?
        .ok_or_else(|| Error::TaskNotFound { id: query.id })
}

pub async fn get_tasks(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    query: GetTasksQuery,
) -> Result<Vec<Task>> {
    let tasks = if query.active_only {
        task_repo.get_active_tasks().await?
    } else {
        task_repo.get_all().await?
    };
    
    let mut filtered_tasks = tasks;
    
    if let Some(status) = query.status {
        filtered_tasks = filtered_tasks
            .into_iter()
            .filter(|task| task.status == status)
            .collect();
    }
    
    if let Some(tags) = query.tags {
        filtered_tasks = filtered_tasks
            .into_iter()
            .filter(|task| {
                tags.iter().any(|tag| task.tags.contains(tag))
            })
            .collect();
    }
    
    Ok(filtered_tasks)
}

pub async fn get_task_by_tags(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    tags: Vec<String>,
) -> Result<Vec<Task>> {
    task_repo.get_by_tags(&tags).await
}

pub async fn get_tasks_by_status(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    status: TaskStatus,
) -> Result<Vec<Task>> {
    task_repo.get_by_status(status).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{TaskStatus, TaskBuilder};
    use domain::InMemoryTaskRepository;

    async fn setup_with_tasks() -> (Arc<dyn TaskRepository + Send + Sync>, Vec<Task>) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        
        let task1 = TaskBuilder::with_name_and_sessions("Work Task".to_string(), 4)
            .with_tags(vec!["work".to_string(), "urgent".to_string()])
            .build().unwrap();
        
        let task2 = TaskBuilder::with_name_and_sessions("Personal Task".to_string(), 2)
            .with_tags(vec!["personal".to_string()])
            .build().unwrap();
            
        let mut task3 = Task::new("Completed Task".to_string(), 1).unwrap();
        task3.increment_session().unwrap(); // Mark as completed
        
        task_repo.create(task1.clone()).await.unwrap();
        task_repo.create(task2.clone()).await.unwrap();
        task_repo.create(task3.clone()).await.unwrap();
        
        (task_repo, vec![task1, task2, task3])
    }

    #[tokio::test]
    async fn should_get_task_by_id() {
        let (task_repo, tasks) = setup_with_tasks().await;
        
        let query = GetTaskQuery {
            id: tasks[0].id.to_string(),
        };
        
        let task = get_task(&task_repo, query).await.unwrap();
        
        assert_eq!(task.name, "Work Task");
        assert_eq!(task.id, tasks[0].id);
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (task_repo, _) = setup_with_tasks().await;
        
        let query = GetTaskQuery {
            id: "nonexistent-id".to_string(),
        };
        
        let result = get_task(&task_repo, query).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn should_get_all_tasks() {
        let (task_repo, _) = setup_with_tasks().await;
        
        let query = GetTasksQuery {
            tags: None,
            status: None,
            active_only: false,
        };
        
        let tasks = get_tasks(&task_repo, query).await.unwrap();
        
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn should_get_active_tasks_only() {
        let (task_repo, _) = setup_with_tasks().await;
        
        let query = GetTasksQuery {
            tags: None,
            status: None,
            active_only: true,
        };
        
        let tasks = get_tasks(&task_repo, query).await.unwrap();
        
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().all(|t| matches!(t.status, TaskStatus::Active | TaskStatus::Queued)));
    }

    #[tokio::test]
    async fn should_filter_tasks_by_tags() {
        let (task_repo, _) = setup_with_tasks().await;
        
        let query = GetTasksQuery {
            tags: Some(vec!["work".to_string()]),
            status: None,
            active_only: false,
        };
        
        let tasks = get_tasks(&task_repo, query).await.unwrap();
        
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Work Task");
    }

    #[tokio::test]
    async fn should_filter_tasks_by_status() {
        let (task_repo, _) = setup_with_tasks().await;
        
        let query = GetTasksQuery {
            tags: None,
            status: Some(TaskStatus::Completed),
            active_only: false,
        };
        
        let tasks = get_tasks(&task_repo, query).await.unwrap();
        
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Completed Task");
    }

    #[tokio::test]
    async fn should_get_tasks_by_tags_using_domain_method() {
        let (task_repo, _) = setup_with_tasks().await;
        
        let tasks = get_task_by_tags(&task_repo, vec!["personal".to_string()]).await.unwrap();
        
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Personal Task");
    }

    #[tokio::test]
    async fn should_get_tasks_by_status_using_domain_method() {
        let (task_repo, _) = setup_with_tasks().await;
        
        let tasks = get_tasks_by_status(&task_repo, TaskStatus::Completed).await.unwrap();
        
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Completed Task");
    }
}