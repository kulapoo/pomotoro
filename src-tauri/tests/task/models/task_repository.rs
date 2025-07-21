use pomotoro_lib::task::{InMemoryTaskRepository, TaskRepositoryTrait, TaskError};
use pomotoro_lib::task::models::Task;
use pomotoro_domain::TaskId;
use std::sync::Arc;

pub struct TaskTestRepository {
    inner: Arc<InMemoryTaskRepository>,
}

impl TaskTestRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InMemoryTaskRepository::new()),
        }
    }

    pub fn with_default_task() -> Self {
        Self {
            inner: Arc::new(InMemoryTaskRepository::with_default_task()),
        }
    }

    pub fn empty() -> Self {
        Self {
            inner: Arc::new(InMemoryTaskRepository::empty()),
        }
    }

    pub async fn seed_with_test_data(&self) -> Result<Vec<TaskId>, Box<dyn std::error::Error>> {
        let mut task_ids = Vec::new();

        let work_task = Task::new("Work Project".to_string(), 4)?
            .with_tags(vec!["work".to_string(), "project".to_string()])
            .with_description("Important work project".to_string());
        self.create(work_task.clone()).await?;
        task_ids.push(work_task.id);

        let study_task = Task::new("Study Session".to_string(), 3)?
            .with_tags(vec!["study".to_string(), "learning".to_string()]);
        self.create(study_task.clone()).await?;
        task_ids.push(study_task.id);

        let mut completed_task = Task::new("Completed Task".to_string(), 2)?;
        completed_task.increment_session()?;
        completed_task.increment_session()?;
        self.create(completed_task.clone()).await?;
        task_ids.push(completed_task.id);

        Ok(task_ids)
    }
}

#[async_trait::async_trait]
impl TaskRepositoryTrait for TaskTestRepository {
    async fn create(&self, task: Task) -> Result<(), TaskError> {
        self.inner.create(task).await
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>, TaskError> {
        self.inner.get_by_id(id).await
    }

    async fn get_all(&self) -> Result<Vec<Task>, TaskError> {
        self.inner.get_all().await
    }

    async fn update(&self, task: Task) -> Result<(), TaskError> {
        self.inner.update(task).await
    }

    async fn delete(&self, id: TaskId) -> Result<bool, TaskError> {
        self.inner.delete(id).await
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>, TaskError> {
        self.inner.get_by_tags(tags).await
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>, TaskError> {
        self.inner.get_active_tasks().await
    }
}