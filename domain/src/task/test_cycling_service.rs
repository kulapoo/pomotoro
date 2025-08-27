use super::{
    Task,
    cycling_service::{CyclerService, CyclingStrategy, DefaultCyclingService},
    id::Id,
    repository::Repository,
};
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Test implementation of CyclerService for use in tests
pub struct TestCyclingService {
    task_repo: Arc<dyn Repository + Send + Sync>,
    domain_service: DefaultCyclingService,
    strategy: CyclingStrategy,
}

impl TestCyclingService {
    pub fn new(
        task_repo: Arc<dyn Repository + Send + Sync>,
        strategy: CyclingStrategy,
    ) -> Self {
        Self {
            task_repo,
            domain_service: DefaultCyclingService::new(),
            strategy,
        }
    }
}

#[async_trait]
impl CyclerService for TestCyclingService {
    async fn get_next_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>> {
        let tasks = self.task_repo.get_active_tasks().await?;
        let available_tasks =
            self.domain_service.filter_available_tasks(&tasks);

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

    async fn validate_task_switch(&self, task_id: Id) -> Result<Option<Task>> {
        if let Some(task) = self.task_repo.get_by_id(task_id).await? {
            self.domain_service.can_switch_to_task(&task)?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    async fn get_active_task_queue(&self) -> Result<Vec<Task>> {
        let tasks = self.task_repo.get_active_tasks().await?;
        let available_tasks =
            self.domain_service.filter_available_tasks(&tasks);
        Ok(available_tasks)
    }

    async fn cycle_to_next_active_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>> {
        self.get_next_task(current_task_id).await
    }

    async fn get_previous_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>> {
        let tasks = self.task_repo.get_active_tasks().await?;
        let available_tasks = self.domain_service.filter_available_tasks(&tasks);
        
        if let Some(prev_task) = self.domain_service.find_previous_task_round_robin(
            &available_tasks,
            current_task_id,
        ) {
            Ok(Some(prev_task.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_incomplete_task_queue(&self) -> Result<Vec<Task>> {
        let tasks = self.task_repo.get_incomplete_tasks().await?;
        Ok(tasks)
    }

    async fn cycle_to_next_incomplete_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>> {
        let tasks = self.task_repo.get_incomplete_tasks().await?;
        
        if let Some(next_task) = self.domain_service.find_next_task_round_robin(
            &tasks,
            current_task_id,
        ) {
            Ok(Some(next_task.clone()))
        } else {
            Ok(None)
        }
    }

    async fn cycle_to_previous_incomplete_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>> {
        let tasks = self.task_repo.get_incomplete_tasks().await?;
        
        if let Some(prev_task) = self.domain_service.find_previous_task_round_robin(
            &tasks,
            current_task_id,
        ) {
            Ok(Some(prev_task.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_task_cycle_position(
        &self,
        task_id: Id,
    ) -> Result<(usize, usize)> {
        let tasks = self.task_repo.get_incomplete_tasks().await?;
        Ok(self.domain_service.find_task_cycle_position(&tasks, task_id))
    }
}
