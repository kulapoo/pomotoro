use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::models::Task;
use pomotoro_domain::{TaskId, TaskStatus, TaskRepository as DomainTaskRepository, Result, Error, Readable, Writable};
use async_trait::async_trait;

pub type TaskRepositoryArc = Arc<dyn DomainTaskRepository + Send + Sync>;

pub struct InMemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        let mut tasks = HashMap::new();
        
        let default_task = Task::new_default();
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
impl DomainTaskRepository for InMemoryTaskRepository {
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
        Ok(tasks
            .values()
            .filter(|task| matches!(task.status, TaskStatus::Active | TaskStatus::Queued))
            .cloned()
            .collect())
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
        self.exists(id.clone()).await
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
        self.update(updated_entity).await
    }

    async fn delete(&mut self, id: &TaskId) -> Result<bool> {
        self.delete(id.clone()).await
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