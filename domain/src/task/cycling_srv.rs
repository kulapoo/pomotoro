use super::{Task, id::Id};
use crate::Result;
use async_trait::async_trait;

/// Domain contract for task cycling operations
/// Concrete implementations belong in infrastructure layer
#[async_trait]
pub trait CyclerService: Send + Sync {
    async fn get_next_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn validate_task_switch(&self, task_id: Id) -> Result<Option<Task>>;
    async fn get_active_task_queue(&self) -> Result<Vec<Task>>;
    async fn cycle_to_next_active_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn get_previous_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn get_incomplete_task_queue(&self) -> Result<Vec<Task>>;
    async fn cycle_to_next_incomplete_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn cycle_to_previous_incomplete_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn get_task_cycle_position(
        &self,
        task_id: Id,
    ) -> Result<(usize, usize)>;
}

/// Domain value object for task cycling strategies
#[derive(Debug, Clone)]
pub enum CyclingStrategy {
    Manual,
    RoundRobin,
    PriorityBased,
}
