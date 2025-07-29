use crate::{Error, Result, Task, TaskId};
use async_trait::async_trait;

/// Domain contract for task cycling operations
/// Concrete implementations belong in infrastructure layer
#[async_trait]
pub trait TaskCyclerService: Send + Sync {
    async fn get_next_task(&self, current_task_id: Option<TaskId>) -> Result<Option<Task>>;
    async fn validate_task_switch(&self, task_id: TaskId) -> Result<Option<Task>>;
    async fn get_active_task_queue(&self) -> Result<Vec<Task>>;
    async fn cycle_to_next_active_task(
        &self,
        current_task_id: Option<TaskId>,
    ) -> Result<Option<Task>>;
}

/// Domain value object for task cycling strategies
#[derive(Debug, Clone)]
pub enum TaskCyclingStrategy {
    Manual,
    RoundRobin,
    PriorityBased,
}

/// Pure domain service for task cycling logic
/// Contains only business rules, no I/O operations
pub struct DefaultTaskCyclingService;

impl Default for DefaultTaskCyclingService {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultTaskCyclingService {
    pub fn new() -> Self {
        Self
    }

    /// Pure domain logic: Find next task using round-robin strategy
    /// No I/O operations - works with in-memory task slice
    pub fn find_next_task_round_robin<'a>(
        &self,
        tasks: &'a [Task],
        current_task_id: Option<TaskId>,
    ) -> Option<&'a Task> {
        if tasks.is_empty() {
            return None;
        }

        if let Some(current_id) = current_task_id {
            // Find current task position and get next one
            if let Some(current_pos) = tasks.iter().position(|t| t.id == current_id) {
                let next_pos = (current_pos + 1) % tasks.len();
                return Some(&tasks[next_pos]);
            }
        }

        // Return first task if no current task or current task not found
        tasks.first()
    }

    /// Pure domain logic: Filter available tasks
    /// Business rule: only active, non-completed tasks are available
    pub fn filter_available_tasks(&self, tasks: &[Task]) -> Vec<Task> {
        tasks
            .iter()
            .filter(|task| !task.is_completed())
            .cloned()
            .collect()
    }

    /// Pure domain logic: Apply cycling strategy
    pub fn apply_cycling_strategy<'a>(
        &self,
        strategy: &TaskCyclingStrategy,
        tasks: &'a [Task],
        current_task_id: Option<TaskId>,
    ) -> Option<&'a Task> {
        match strategy {
            TaskCyclingStrategy::Manual => {
                // In manual mode, don't automatically cycle
                current_task_id.and_then(|id| tasks.iter().find(|t| t.id == id))
            }
            TaskCyclingStrategy::RoundRobin => {
                self.find_next_task_round_robin(tasks, current_task_id)
            }
            TaskCyclingStrategy::PriorityBased => {
                // TODO: Priority-based cycling - future enhancement
                // For now, fallback to round-robin
                self.find_next_task_round_robin(tasks, current_task_id)
            }
        }
    }

    /// Pure domain logic: Validate task can be switched to
    pub fn can_switch_to_task(&self, task: &Task) -> Result<()> {
        if task.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskDefaults;

    fn create_test_tasks() -> Vec<Task> {
        let defaults = TaskDefaults::default();
        vec![
            Task::new_with_defaults("Task 1".to_string(), 4, &defaults).unwrap(),
            Task::new_with_defaults("Task 2".to_string(), 3, &defaults).unwrap(),
            Task::new_with_defaults("Task 3".to_string(), 2, &defaults).unwrap(),
        ]
    }

    #[test]
    fn should_find_next_task_round_robin() {
        let service = DefaultTaskCyclingService::new();
        let tasks = create_test_tasks();

        // First call with no current task should return first task
        let first_task = service.find_next_task_round_robin(&tasks, None).unwrap();
        assert_eq!(first_task.name, "Task 1");

        // Next call should return second task
        let second_task = service
            .find_next_task_round_robin(&tasks, Some(first_task.id.clone()))
            .unwrap();
        assert_eq!(second_task.name, "Task 2");

        // Third call should return third task
        let third_task = service
            .find_next_task_round_robin(&tasks, Some(second_task.id.clone()))
            .unwrap();
        assert_eq!(third_task.name, "Task 3");

        // Fourth call should wrap around to first task
        let fourth_task = service
            .find_next_task_round_robin(&tasks, Some(third_task.id.clone()))
            .unwrap();
        assert_eq!(fourth_task.name, "Task 1");
    }

    #[test]
    fn should_return_none_for_empty_task_list() {
        let service = DefaultTaskCyclingService::new();
        let empty_tasks: Vec<Task> = vec![];

        let result = service.find_next_task_round_robin(&empty_tasks, None);
        assert!(result.is_none());
    }

    #[test]
    fn should_return_first_task_when_current_not_found() {
        let service = DefaultTaskCyclingService::new();
        let tasks = create_test_tasks();
        let non_existent_id = TaskId::new();

        let result = service
            .find_next_task_round_robin(&tasks, Some(non_existent_id))
            .unwrap();
        assert_eq!(result.name, "Task 1");
    }

    #[test]
    fn should_filter_available_tasks() {
        let service = DefaultTaskCyclingService::new();
        let mut tasks = create_test_tasks();

        // Complete one task
        tasks[1].increment_session().unwrap(); // Complete first session
        tasks[1].increment_session().unwrap(); // Complete second session
        tasks[1].increment_session().unwrap(); // Complete all sessions

        let available = service.filter_available_tasks(&tasks);
        assert_eq!(available.len(), 2);
        assert!(!available.iter().any(|t| t.name == "Task 2"));
    }

    #[test]
    fn should_apply_round_robin_strategy() {
        let service = DefaultTaskCyclingService::new();
        let tasks = create_test_tasks();
        let strategy = TaskCyclingStrategy::RoundRobin;

        let first_task = service
            .apply_cycling_strategy(&strategy, &tasks, None)
            .unwrap();
        assert_eq!(first_task.name, "Task 1");

        let second_task = service
            .apply_cycling_strategy(&strategy, &tasks, Some(first_task.id.clone()))
            .unwrap();
        assert_eq!(second_task.name, "Task 2");
    }

    #[test]
    fn should_apply_manual_strategy() {
        let service = DefaultTaskCyclingService::new();
        let tasks = create_test_tasks();
        let strategy = TaskCyclingStrategy::Manual;

        // In manual mode, should return current task
        let result = service
            .apply_cycling_strategy(&strategy, &tasks, Some(tasks[1].id.clone()))
            .unwrap();
        assert_eq!(result.name, "Task 2");

        // With no current task, should return None
        let result = service.apply_cycling_strategy(&strategy, &tasks, None);
        assert!(result.is_none());
    }

    #[test]
    fn should_validate_task_can_be_switched_to() {
        let service = DefaultTaskCyclingService::new();
        let tasks = create_test_tasks();

        // Active task should be valid
        let result = service.can_switch_to_task(&tasks[0]);
        assert!(result.is_ok());

        // Completed task should be invalid
        let mut completed_task = tasks[1].clone();
        completed_task.increment_session().unwrap(); // Complete first session
        completed_task.increment_session().unwrap(); // Complete second session
        completed_task.increment_session().unwrap(); // Complete all sessions

        let result = service.can_switch_to_task(&completed_task);
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }
}
