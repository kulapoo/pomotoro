use super::{Task, Id};

/// Pure domain trait for task cycling operations.
///
/// This trait contains only pure business logic with no I/O operations.
/// All methods work with in-memory data and return computed results.
///
/// # Design Principles
/// - Pure functions only (no async, no I/O)
/// - Stateless operations (all state passed as parameters)
/// - Minimal interface (only essential methods)
///
/// # Clean Architecture Compliance
/// - Lives in domain layer
/// - No infrastructure dependencies
/// - No async/await keywords
/// - Works with domain entities only
pub trait TaskCycling {
    /// Determines the next task in the cycling sequence.
    ///
    /// # Arguments
    /// * `tasks` - Slice of available tasks
    /// * `current_task_id` - Optional ID of the currently active task
    ///
    /// # Returns
    /// * `Some(Task)` - The next task in the cycle
    /// * `None` - If no tasks are available or all are completed
    fn get_next_task(&self, tasks: &[Task], current_task_id: Option<&Id>) -> Option<Task>;

    /// Checks if there are any active (incomplete) tasks available.
    ///
    /// # Arguments
    /// * `tasks` - Slice of tasks to check
    ///
    /// # Returns
    /// * `true` if at least one incomplete task exists
    /// * `false` if all tasks are completed or list is empty
    fn has_tasks(&self, tasks: &[Task]) -> bool;

    /// Filters tasks to return only active (incomplete) ones.
    ///
    /// # Arguments
    /// * `tasks` - Slice of tasks to filter
    ///
    /// # Returns
    /// * Vector of incomplete tasks
    fn filter_active_tasks(&self, tasks: &[Task]) -> Vec<Task>;
}

/// Default implementation of TaskCycling using round-robin strategy.
///
/// This implementation provides standard cycling behavior:
/// - Cycles through incomplete tasks only
/// - Wraps around to the beginning after the last task
/// - Skips completed tasks automatically
pub struct PureTaskCycling;

impl TaskCycling for PureTaskCycling {
    fn get_next_task(&self, tasks: &[Task], current_task_id: Option<&Id>) -> Option<Task> {
        let active_tasks = self.filter_active_tasks(tasks);

        if active_tasks.is_empty() {
            return None;
        }

        // If no current task, return the first active task
        let Some(current_id) = current_task_id else {
            return active_tasks.into_iter().next();
        };

        // Find current task position in active tasks
        let current_position = active_tasks
            .iter()
            .position(|task| &task.id == current_id);

        match current_position {
            Some(pos) => {
                // Get next task in cycle (wrap around to beginning)
                let next_pos = (pos + 1) % active_tasks.len();
                active_tasks.get(next_pos).cloned()
            }
            None => {
                // Current task not found in active tasks, return first
                active_tasks.into_iter().next()
            }
        }
    }

    fn has_tasks(&self, tasks: &[Task]) -> bool {
        tasks.iter().any(|task| !task.status.is_completed())
    }

    fn filter_active_tasks(&self, tasks: &[Task]) -> Vec<Task> {
        tasks
            .iter()
            .filter(|task| !task.status.is_completed())
            .cloned()
            .collect()
    }
}

/// Extended cycling operations for backward compatibility.
///
/// This trait provides additional methods that were in the original
/// CyclerService but can be derived from the core three methods.
pub trait TaskCyclingExt: TaskCycling {
    /// Gets the previous task in the cycling sequence.
    fn get_previous_task(&self, tasks: &[Task], current_task_id: Option<&Id>) -> Option<Task> {
        let active_tasks = self.filter_active_tasks(tasks);

        if active_tasks.is_empty() {
            return None;
        }

        let Some(current_id) = current_task_id else {
            return active_tasks.last().cloned();
        };

        let current_position = active_tasks
            .iter()
            .position(|task| &task.id == current_id);

        match current_position {
            Some(pos) => {
                let prev_pos = if pos == 0 {
                    active_tasks.len() - 1
                } else {
                    pos - 1
                };
                active_tasks.get(prev_pos).cloned()
            }
            None => active_tasks.last().cloned(),
        }
    }

    /// Gets the position of a task in the cycle.
    fn get_task_position(&self, tasks: &[Task], task_id: &Id) -> (usize, usize) {
        let active_tasks = self.filter_active_tasks(tasks);

        let position = active_tasks
            .iter()
            .position(|task| &task.id == task_id)
            .map(|p| p + 1)
            .unwrap_or(0);

        (position, active_tasks.len())
    }

    /// Filters to get only incomplete tasks.
    fn filter_incomplete_tasks(&self, tasks: &[Task]) -> Vec<Task> {
        // For now, incomplete and active are the same
        // This could change if we add more statuses
        self.filter_active_tasks(tasks)
    }
}

// Blanket implementation for all types that implement TaskCycling
impl<T: TaskCycling> TaskCyclingExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Status, Builder as TaskBuilder};

    fn create_test_task(name: &str, completed: bool) -> Task {
        let mut task = TaskBuilder::with_name_and_sessions(name.to_string(), 4)
            .build()
            .unwrap();

        if completed {
            task.status = Status::Completed;
        }

        task
    }

    #[test]
    fn test_get_next_task_empty_list() {
        let cycling = PureTaskCycling;
        let tasks = vec![];

        assert!(cycling.get_next_task(&tasks, None).is_none());
    }

    #[test]
    fn test_get_next_task_all_completed() {
        let cycling = PureTaskCycling;
        let tasks = vec![
            create_test_task("Task 1", true),
            create_test_task("Task 2", true),
        ];

        assert!(cycling.get_next_task(&tasks, None).is_none());
    }

    #[test]
    fn test_get_next_task_cycles_correctly() {
        let cycling = PureTaskCycling;
        let tasks = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", true),  // Completed - should be skipped
            create_test_task("Task 3", false),
        ];

        // First task when no current task
        let next = cycling.get_next_task(&tasks, None);
        assert_eq!(next.unwrap().name, "Task 1");

        // Cycle from Task 1 to Task 3 (skip Task 2)
        let next = cycling.get_next_task(&tasks, Some(&tasks[0].id));
        assert_eq!(next.unwrap().name, "Task 3");

        // Cycle from Task 3 back to Task 1
        let next = cycling.get_next_task(&tasks, Some(&tasks[2].id));
        assert_eq!(next.unwrap().name, "Task 1");
    }

    #[test]
    fn test_has_tasks() {
        let cycling = PureTaskCycling;

        let empty = vec![];
        assert!(!cycling.has_tasks(&empty));

        let all_completed = vec![
            create_test_task("Task 1", true),
            create_test_task("Task 2", true),
        ];
        assert!(!cycling.has_tasks(&all_completed));

        let has_active = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", true),
        ];
        assert!(cycling.has_tasks(&has_active));
    }

    #[test]
    fn test_filter_active_tasks() {
        let cycling = PureTaskCycling;
        let tasks = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", true),
            create_test_task("Task 3", false),
            create_test_task("Task 4", true),
        ];

        let active = cycling.filter_active_tasks(&tasks);
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].name, "Task 1");
        assert_eq!(active[1].name, "Task 3");
    }

    #[test]
    fn test_get_previous_task() {
        let cycling = PureTaskCycling;
        let tasks = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", false),
            create_test_task("Task 3", false),
        ];

        // Previous from Task 1 should be Task 3
        let prev = cycling.get_previous_task(&tasks, Some(&tasks[0].id));
        assert_eq!(prev.unwrap().name, "Task 3");

        // Previous from Task 3 should be Task 2
        let prev = cycling.get_previous_task(&tasks, Some(&tasks[2].id));
        assert_eq!(prev.unwrap().name, "Task 2");

        // No current task should return last
        let prev = cycling.get_previous_task(&tasks, None);
        assert_eq!(prev.unwrap().name, "Task 3");
    }

    #[test]
    fn test_get_task_position() {
        let cycling = PureTaskCycling;
        let tasks = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", true),  // Completed - not counted
            create_test_task("Task 3", false),
        ];

        let (pos, total) = cycling.get_task_position(&tasks, &tasks[0].id);
        assert_eq!(pos, 1);
        assert_eq!(total, 2);

        let (pos, total) = cycling.get_task_position(&tasks, &tasks[2].id);
        assert_eq!(pos, 2);
        assert_eq!(total, 2);

        // Unknown task
        let unknown_id = Id::new();
        let (pos, total) = cycling.get_task_position(&tasks, &unknown_id);
        assert_eq!(pos, 0);
        assert_eq!(total, 2);
    }
}