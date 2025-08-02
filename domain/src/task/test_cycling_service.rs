use crate::{Result, Task, TaskId, TaskRepository, TaskCyclerService, DefaultTaskCyclingService, TaskCyclingStrategy};
use async_trait::async_trait;
use std::sync::Arc;

/// Test implementation of TaskCyclerService for use in tests
pub struct TestTaskCyclingService {
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    domain_service: DefaultTaskCyclingService,
    strategy: TaskCyclingStrategy,
}

impl TestTaskCyclingService {
    pub fn new(
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
        strategy: TaskCyclingStrategy,
    ) -> Self {
        Self {
            task_repo,
            domain_service: DefaultTaskCyclingService::new(),
            strategy,
        }
    }
}

#[async_trait]
impl TaskCyclerService for TestTaskCyclingService {
    async fn get_next_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>> {
        let tasks = self.task_repo.get_active_tasks().await?;
        let available_tasks = self.domain_service.filter_available_tasks(&tasks);
        
        if let Some(next_task) = self.domain_service.apply_cycling_strategy(
            &self.strategy,
            &available_tasks,
            current_task_id,
        ) {
            Ok(Some(next_task.clone()))
        } else {
            Ok(None)
        }
    }

    async fn validate_task_switch(&self, task_id: TaskId) -> Result<Option<Task>> {
        if let Some(task) = self.task_repo.get_by_id(task_id).await? {
            self.domain_service.can_switch_to_task(&task)?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    async fn get_active_task_queue(&self) -> Result<Vec<Task>> {
        let tasks = self.task_repo.get_active_tasks().await?;
        let available_tasks = self.domain_service.filter_available_tasks(&tasks);
        Ok(available_tasks)
    }

    async fn cycle_to_next_active_task(
        &self,
        current_task_id: Option<TaskId>,
    ) -> Result<Option<Task>> {
        self.get_next_task(current_task_id).await
    }
}