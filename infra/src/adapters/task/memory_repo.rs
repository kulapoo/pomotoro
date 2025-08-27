use async_trait::async_trait;
use domain::{
    Error, Readable, Result, Task, TaskBuilder,
    TaskDefaults, TaskId, TaskRepository, TaskStatus, Writable,
    shared_kernel::traits::searchable::{SearchCriteria, Searchable},
    task::repository::{SearchOptions, SortBy, SortOrder},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type TaskRepositoryArc = Arc<dyn TaskRepository + Send + Sync>;

// InMemoryTaskRepository stores domain objects directly in memory
// For file/database persistence, use TaskDto for serialization
pub struct InMemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
}

impl Default for InMemoryTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        let mut tasks = HashMap::new();

        let default_task =
            Task::new_default().expect("Default task creation should not fail");
        tasks.insert(default_task.id, default_task);

        Self {
            tasks: Arc::new(RwLock::new(tasks)),
        }
    }

    pub fn empty() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_default_task(defaults: &TaskDefaults) -> Self {
        let mut tasks = HashMap::new();

        // Use the builder to create a proper default task
        let default_task = TaskBuilder::default_task()
            .build_with_defaults(defaults)
            .expect("Default task creation should not fail");

        tasks.insert(default_task.id, default_task);

        Self {
            tasks: Arc::new(RwLock::new(tasks)),
        }
    }

    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        let mut task_map = HashMap::new();
        for task in tasks {
            let task_id = task.id;
            task_map.insert(task_id, task);
        }
        Self {
            tasks: Arc::new(RwLock::new(task_map)),
        }
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn create(&self, task: Task) -> Result<()> {
        let mut tasks =
            self.tasks.write().map_err(|e| Error::RepositoryError {
                message: format!("Lock error: {e}"),
            })?;

        if tasks.contains_key(&task.id) {
            return Err(Error::TaskNotFound {
                id: task.id.to_string(),
            });
        }

        let task_id = task.id;
        tasks.insert(task_id, task);
        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(task_list)
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        let mut active_tasks: Vec<Task> = tasks
            .values()
            .filter(|task| {
                matches!(task.status, TaskStatus::Active | TaskStatus::Queued)
            })
            .cloned()
            .collect();
        // Sort by creation time for consistent ordering
        active_tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(active_tasks)
    }

    async fn update(&self, task: Task) -> Result<()> {
        let mut tasks =
            self.tasks.write().map_err(|e| Error::RepositoryError {
                message: format!("Lock error: {e}"),
            })?;

        if !tasks.contains_key(&task.id) {
            return Err(Error::TaskNotFound {
                id: task.id.to_string(),
            });
        }

        let task_id = task.id;
        tasks.insert(task_id, task);
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool> {
        let mut tasks =
            self.tasks.write().map_err(|e| Error::RepositoryError {
                message: format!("Lock error: {e}"),
            })?;

        if let Some(task) = tasks.get(&id) {
            if task.is_default() {
                return Ok(false); // Prevent deletion of default task
            }
        }

        Ok(tasks.remove(&id).is_some())
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        Ok(tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.tags.contains(tag)))
            .cloned()
            .collect())
    }

    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        Ok(tasks
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    async fn exists(&self, id: TaskId) -> Result<bool> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        Ok(tasks.contains_key(&id))
    }

    async fn get_default_task(&self) -> Result<Option<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        Ok(tasks.values().find(|task| task.default).cloned())
    }

    async fn search(&self, options: SearchOptions) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        
        let mut result: Vec<Task> = tasks.values()
            .filter(|task| {
                let mut matches = true;
                
                if let Some(ref query) = options.criteria.query {
                    let query_lower = query.to_lowercase();
                    let name_matches = task.name.to_lowercase().contains(&query_lower);
                    let description_matches = task.description
                        .as_ref()
                        .map_or(false, |desc| desc.to_lowercase().contains(&query_lower));
                    let tag_matches = task.tags.iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower));
                    matches = matches && (name_matches || description_matches || tag_matches);
                }
                
                if let Some(ref tags) = options.criteria.tags {
                    let has_any_tag = tags.iter()
                        .any(|tag| task.tags.contains(tag));
                    matches = matches && has_any_tag;
                }
                
                if let Some(ref status_str) = options.criteria.status {
                    let status_matches = match status_str.to_lowercase().as_str() {
                        "active" => task.status == TaskStatus::Active,
                        "completed" => task.status == TaskStatus::Completed,
                        "paused" => task.status == TaskStatus::Paused,
                        "queued" => task.status == TaskStatus::Queued,
                        _ => false,
                    };
                    matches = matches && status_matches;
                }
                
                matches
            })
            .cloned()
            .collect();
        
        if let Some(sort_by) = options.sort_by {
            let sort_order = options.sort_order.unwrap_or(SortOrder::Ascending);
            result.sort_by(|a, b| {
                let ordering = match sort_by {
                    SortBy::Name => a.name.cmp(&b.name),
                    SortBy::CreatedAt => a.created_at.cmp(&b.created_at),
                    SortBy::SessionsCompleted => a.current_sessions.cmp(&b.current_sessions),
                    SortBy::Status => {
                        let status_order = |s: &TaskStatus| match s {
                            TaskStatus::Active => 0,
                            TaskStatus::Queued => 1,
                            TaskStatus::Paused => 2,
                            TaskStatus::Completed => 3,
                        };
                        status_order(&a.status).cmp(&status_order(&b.status))
                    },
                };
                match sort_order {
                    SortOrder::Ascending => ordering,
                    SortOrder::Descending => ordering.reverse(),
                }
            });
        }
        
        if let Some(limit) = options.criteria.limit {
            let offset = options.criteria.offset.unwrap_or(0);
            result = result.into_iter().skip(offset).take(limit).collect();
        }
        
        Ok(result)
    }
    
    async fn search_fuzzy(&self, query: &str) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        
        let query_lower = query.to_lowercase();
        let query_parts: Vec<&str> = query_lower.split_whitespace().collect();
        
        let mut scored_tasks: Vec<(Task, usize)> = tasks.values()
            .map(|task| {
                let mut score = 0;
                
                for part in &query_parts {
                    if task.name.to_lowercase().contains(part) {
                        score += 3;
                    }
                    if let Some(ref desc) = task.description {
                        if desc.to_lowercase().contains(part) {
                            score += 2;
                        }
                    }
                    for tag in &task.tags {
                        if tag.to_lowercase().contains(part) {
                            score += 1;
                        }
                    }
                }
                
                (task.clone(), score)
            })
            .filter(|(_, score)| *score > 0)
            .collect();
        
        scored_tasks.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.created_at.cmp(&b.0.created_at)));
        
        Ok(scored_tasks.into_iter().map(|(task, _)| task).collect())
    }
    
    async fn get_incomplete_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        
        Ok(tasks
            .values()
            .filter(|task| !task.is_completed())
            .cloned()
            .collect())
    }
    
    async fn get_completed_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        
        Ok(tasks
            .values()
            .filter(|task| task.is_completed())
            .cloned()
            .collect())
    }
}

#[async_trait]
impl Readable<Task, TaskId> for InMemoryTaskRepository {
    async fn find_by_id(&self, id: &TaskId) -> Result<Option<Task>> {
        self.get_by_id(*id).await
    }

    async fn find_all(&self) -> Result<Vec<Task>> {
        self.get_all().await
    }

    async fn exists(&self, id: &TaskId) -> Result<bool> {
        TaskRepository::exists(self, *id).await
    }

    async fn count(&self) -> Result<usize> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError {
            message: format!("Lock error: {e}"),
        })?;
        Ok(tasks.len())
    }
}

#[async_trait]
impl Searchable<Task> for InMemoryTaskRepository {
    async fn search(&self, criteria: &SearchCriteria) -> Result<Vec<Task>> {
        let options = SearchOptions {
            criteria: criteria.clone(),
            sort_by: None,
            sort_order: None,
        };
        TaskRepository::search(self, options).await
    }
    
    async fn search_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        self.get_by_tags(tags).await
    }
    
    async fn search_by_query(&self, query: &str) -> Result<Vec<Task>> {
        self.search_fuzzy(query).await
    }
}

#[async_trait]
impl Writable<Task, TaskId> for InMemoryTaskRepository {
    async fn save(&mut self, entity: &Task) -> Result<()> {
        self.create(entity.clone()).await
    }

    async fn update(&mut self, id: &TaskId, entity: &Task) -> Result<()> {
        let mut updated_entity = entity.clone();
        updated_entity.id = *id;
        TaskRepository::update(self, updated_entity).await
    }

    async fn delete(&mut self, id: &TaskId) -> Result<bool> {
        TaskRepository::delete(self, *id).await
    }

    async fn delete_all(&mut self) -> Result<usize> {
        let mut tasks =
            self.tasks.write().map_err(|e| Error::RepositoryError {
                message: format!("Lock error: {e}"),
            })?;
        let count = tasks.len();
        tasks.clear();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::TaskBuilder;

    #[tokio::test]
    async fn should_create_and_retrieve_task() {
        let repo = InMemoryTaskRepository::new();
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        let task_id = task.id;

        repo.create(task.clone()).await.unwrap();
        let retrieved = repo.get_by_id(task_id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Task");
    }

    #[tokio::test]
    async fn should_update_existing_task() {
        let repo = InMemoryTaskRepository::new();
        let mut task = Task::new("Original".to_string(), 4).unwrap();
        let task_id = task.id;

        repo.create(task.clone()).await.unwrap();

        task.name = "Updated".to_string();
        repo.update(task).await.unwrap();

        let retrieved = repo.get_by_id(task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated");
    }

    #[tokio::test]
    async fn should_filter_tasks_by_status() {
        let active_task = Task::new("Active".to_string(), 4).unwrap();
        let mut completed_task = Task::new("Completed".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap(); // Makes it completed

        let repo = InMemoryTaskRepository::with_tasks(vec![
            active_task.clone(),
            completed_task,
        ]);

        let active_tasks = repo.get_active_tasks().await.unwrap();
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].name, "Active");
    }

    #[tokio::test]
    async fn should_filter_tasks_by_tags() {
        let work_task =
            TaskBuilder::with_name_and_sessions("Work Task".to_string(), 4)
                .with_tags(vec!["work".to_string(), "urgent".to_string()])
                .build()
                .unwrap();

        let personal_task =
            TaskBuilder::with_name_and_sessions("Personal Task".to_string(), 2)
                .with_tags(vec!["personal".to_string()])
                .build()
                .unwrap();

        let repo =
            InMemoryTaskRepository::with_tasks(vec![work_task, personal_task]);

        let work_tasks = repo.get_by_tags(&["work".to_string()]).await.unwrap();
        assert_eq!(work_tasks.len(), 1);
        assert_eq!(work_tasks[0].name, "Work Task");
    }
}
