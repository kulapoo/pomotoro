use super::{Task, id::Id};
use crate::Result;
use async_trait::async_trait;

/// Domain contract for task cycling operations
/// Concrete implementations belong in infrastructure layer
///
/// ARCHITECTURE VIOLATION: This trait contains async methods which violate
/// domain purity. Domain layer should not have I/O operations.
///
/// TODO: Migration Plan (v2):
/// 1. Remove async/await from all methods
/// 2. Reduce to 3 essential methods: get_next_task, has_tasks, filter_active_tasks
/// 3. Move I/O operations to infrastructure layer only
/// 4. Use AutoCycleService for pure domain logic instead
///
/// For now, AutoCycleService provides the pure domain logic while this
/// trait remains for backward compatibility.
#[async_trait]
pub trait CyclerService: Send + Sync {
    async fn get_next_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn get_previous_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn get_default_task(&self) -> Result<Option<Task>>;
    async fn get_cycle_tasks(&self) -> Result<Vec<Task>>;
    async fn has_tasks(&self) -> Result<bool>;
    
    // Methods for cycling through active tasks
    async fn cycle_to_next_active_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn get_active_task_queue(&self) -> Result<Vec<Task>>;
    
    // Methods for cycling through incomplete tasks
    async fn cycle_to_next_incomplete_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn cycle_to_previous_incomplete_task(
        &self,
        current_task_id: Option<Id>,
    ) -> Result<Option<Task>>;
    async fn get_incomplete_task_queue(&self) -> Result<Vec<Task>>;
    async fn get_task_cycle_position(&self, task_id: Id) -> Result<(usize, usize)>;
    
    // Task validation
    async fn validate_task_switch(&self, task_id: Id) -> Result<()>;
}
