use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use domain::{Task, TaskId, TaskStatus, TaskRepository, Result, Error, Readable, Writable};
use async_trait::async_trait;

pub type TaskRepositoryArc = Arc<dyn TaskRepository + Send + Sync>;

// InMemoryTaskRepository stores domain objects directly in memory
// For file/database persistence, use TaskDto for serialization
pub struct InMemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        let mut tasks = HashMap::new();
        
        let default_task = Task::new_default().expect("Default task creation should not fail");
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

    pub fn with_default_task() -> Self {
        Self::new()
    }

    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        let mut task_map = HashMap::new();
        for task in tasks {
            let task_id = task.id.clone();
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
        let mut tasks = self.tasks.write().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        
        if tasks.contains_key(&task.id) {
            return Err(Error::TaskNotFound { 
                id: task.id.to_string() 
            });
        }
        
        let task_id = task.id.clone();
        tasks.insert(task_id, task);
        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(task_list)
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        let mut active_tasks: Vec<Task> = tasks
            .values()
            .filter(|task| matches!(task.status, TaskStatus::Active | TaskStatus::Queued))
            .cloned()
            .collect();
        // Sort by creation time for consistent ordering
        active_tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(active_tasks)
    }

    async fn update(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.write().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        
        if !tasks.contains_key(&task.id) {
            return Err(Error::TaskNotFound { 
                id: task.id.to_string() 
            });
        }
        
        let task_id = task.id.clone();
        tasks.insert(task_id, task);
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool> {
        let mut tasks = self.tasks.write().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        
        // Check if this is the default task and prevent deletion
        if let Some(task) = tasks.get(&id) {
            if task.name == "Focus Session" && 
               task.description == Some("Default pomodoro task for focused work".to_string()) {
                return Ok(false); // Prevent deletion of default task
            }
        }
        
        Ok(tasks.remove(&id).is_some())
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        Ok(tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.tags.contains(tag)))
            .cloned()
            .collect())
    }

    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        Ok(tasks
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    async fn exists(&self, id: TaskId) -> Result<bool> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        Ok(tasks.contains_key(&id))
    }
}

#[async_trait]
impl Readable<Task, TaskId> for InMemoryTaskRepository {
    async fn find_by_id(&self, id: &TaskId) -> Result<Option<Task>> {
        self.get_by_id(id.clone()).await
    }

    async fn find_all(&self) -> Result<Vec<Task>> {
        self.get_all().await
    }

    async fn exists(&self, id: &TaskId) -> Result<bool> {
        TaskRepository::exists(self, id.clone()).await
    }

    async fn count(&self) -> Result<usize> {
        let tasks = self.tasks.read().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
        })?;
        Ok(tasks.len())
    }
}

#[async_trait]
impl Writable<Task, TaskId> for InMemoryTaskRepository {
    async fn save(&mut self, entity: &Task) -> Result<()> {
        self.create(entity.clone()).await
    }

    async fn update(&mut self, id: &TaskId, entity: &Task) -> Result<()> {
        let mut updated_entity = entity.clone();
        updated_entity.id = id.clone();
        TaskRepository::update(self, updated_entity).await
    }

    async fn delete(&mut self, id: &TaskId) -> Result<bool> {
        TaskRepository::delete(self, id.clone()).await
    }

    async fn delete_all(&mut self) -> Result<usize> {
        let mut tasks = self.tasks.write().map_err(|e| Error::RepositoryError { 
            message: format!("Lock error: {}", e) 
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
        let task_id = task.id.clone();

        repo.create(task.clone()).await.unwrap();
        let retrieved = repo.get_by_id(task_id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Task");
    }

    #[tokio::test]
    async fn should_update_existing_task() {
        let repo = InMemoryTaskRepository::new();
        let mut task = Task::new("Original".to_string(), 4).unwrap();
        let task_id = task.id.clone();

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
        let work_task = TaskBuilder::with_name_and_sessions("Work Task".to_string(), 4)
            .with_tags(vec!["work".to_string(), "urgent".to_string()])
            .build().unwrap();
        
        let personal_task = TaskBuilder::with_name_and_sessions("Personal Task".to_string(), 2)
            .with_tags(vec!["personal".to_string()])
            .build().unwrap();

        let repo = InMemoryTaskRepository::with_tasks(vec![
            work_task,
            personal_task,
        ]);

        let work_tasks = repo.get_by_tags(&["work".to_string()]).await.unwrap();
        assert_eq!(work_tasks.len(), 1);
        assert_eq!(work_tasks[0].name, "Work Task");
    }
}