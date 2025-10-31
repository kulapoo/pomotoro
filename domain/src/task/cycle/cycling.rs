use crate::{Task, TaskId};

/// Determines the next task in a cycling sequence using round-robin strategy.
///
/// This pure domain trait contains only business logic with no I/O operations.
/// All methods work with in-memory data and return computed results.
#[allow(dead_code)]
pub trait Cycling {
    /// Gets the next task in the cycling sequence.
    ///
    /// Returns `None` if no tasks are available or all are completed.
    fn next_task(&self, tasks: &[Task], current_task_id: Option<&TaskId>) -> Option<Task>;

    /// Gets the previous task in the cycling sequence.
    ///
    /// Returns `None` if no tasks are available or all are completed.
    fn previous_task(&self, tasks: &[Task], current_task_id: Option<&TaskId>) -> Option<Task>;

    /// Checks if there are any active (incomplete) tasks.
    fn has_active_tasks(&self, tasks: &[Task]) -> bool;
}

/// Default round-robin cycling implementation.
pub struct RoundRobinCycling;

impl Cycling for RoundRobinCycling {
    fn next_task(&self, tasks: &[Task], current_task_id: Option<&TaskId>) -> Option<Task> {
        let active_tasks = self.active_tasks(tasks);

        if active_tasks.is_empty() {
            return None;
        }

        let current_position = current_task_id
            .and_then(|id| active_tasks.iter().position(|task| &task.id == id));

        match current_position {
            Some(pos) => {
                let next_pos = (pos + 1) % active_tasks.len();
                active_tasks.get(next_pos).cloned()
            }
            None => active_tasks.first().cloned(),
        }
    }

    fn previous_task(&self, tasks: &[Task], current_task_id: Option<&TaskId>) -> Option<Task> {
        let active_tasks = self.active_tasks(tasks);

        if active_tasks.is_empty() {
            return None;
        }

        let current_position = current_task_id
            .and_then(|id| active_tasks.iter().position(|task| &task.id == id));

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

    fn has_active_tasks(&self, tasks: &[Task]) -> bool {
        tasks.iter().any(|task| !task.status.is_completed())
    }
}

impl RoundRobinCycling {
    /// Returns only active (incomplete) tasks.
    fn active_tasks(&self, tasks: &[Task]) -> Vec<Task> {
        tasks
            .iter()
            .filter(|task| !task.status.is_completed())
            .cloned()
            .collect()
    }

    /// Gets the position of a task among active tasks.
    ///
    /// Returns `(position, total_active)` where position is 1-based.
    /// Returns `(0, total_active)` if the task is not found or completed.
    pub fn task_position(&self, tasks: &[Task], task_id: &TaskId) -> (usize, usize) {
        let active_tasks = self.active_tasks(tasks);
        let total = active_tasks.len();

        let position = active_tasks
            .iter()
            .position(|task| &task.id == task_id)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        (position, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Status, Builder as TaskBuilder};

    fn create_task(name: &str, completed: bool) -> Task {
        let mut task = TaskBuilder::with_name_and_sessions(name.to_string(), 4)
            .build()
            .unwrap();

        if completed {
            task.status = Status::Completed;
        }

        task
    }

    #[test]
    fn test_next_task_empty_list() {
        let cycling = RoundRobinCycling;
        assert!(cycling.next_task(&[], None).is_none());
    }

    #[test]
    fn test_next_task_all_completed() {
        let cycling = RoundRobinCycling;
        let tasks = vec![
            create_task("Task 1", true),
            create_task("Task 2", true),
        ];

        assert!(cycling.next_task(&tasks, None).is_none());
    }

    #[test]
    fn test_next_task_cycles_correctly() {
        let cycling = RoundRobinCycling;
        let tasks = vec![
            create_task("Task 1", false),
            create_task("Task 2", true),  // Completed - skipped
            create_task("Task 3", false),
        ];

        // First task when no current task
        let next = cycling.next_task(&tasks, None).unwrap();
        assert_eq!(next.name, "Task 1");

        // Cycle from Task 1 to Task 3 (skip Task 2)
        let next = cycling.next_task(&tasks, Some(&tasks[0].id)).unwrap();
        assert_eq!(next.name, "Task 3");

        // Cycle from Task 3 back to Task 1
        let next = cycling.next_task(&tasks, Some(&tasks[2].id)).unwrap();
        assert_eq!(next.name, "Task 1");
    }

    #[test]
    fn test_previous_task() {
        let cycling = RoundRobinCycling;
        let tasks = vec![
            create_task("Task 1", false),
            create_task("Task 2", false),
            create_task("Task 3", false),
        ];

        // Previous from Task 1 should be Task 3
        let prev = cycling.previous_task(&tasks, Some(&tasks[0].id)).unwrap();
        assert_eq!(prev.name, "Task 3");

        // Previous from Task 3 should be Task 2
        let prev = cycling.previous_task(&tasks, Some(&tasks[2].id)).unwrap();
        assert_eq!(prev.name, "Task 2");

        // No current task should return last
        let prev = cycling.previous_task(&tasks, None).unwrap();
        assert_eq!(prev.name, "Task 3");
    }

    #[test]
    fn test_has_active_tasks() {
        let cycling = RoundRobinCycling;

        assert!(!cycling.has_active_tasks(&[]));

        let all_completed = vec![
            create_task("Task 1", true),
            create_task("Task 2", true),
        ];
        assert!(!cycling.has_active_tasks(&all_completed));

        let has_active = vec![
            create_task("Task 1", false),
            create_task("Task 2", true),
        ];
        assert!(cycling.has_active_tasks(&has_active));
    }

    #[test]
    fn test_task_position() {
        let cycling = RoundRobinCycling;
        let tasks = vec![
            create_task("Task 1", false),
            create_task("Task 2", true),  // Completed - not counted
            create_task("Task 3", false),
        ];

        let (pos, total) = cycling.task_position(&tasks, &tasks[0].id);
        assert_eq!(pos, 1);
        assert_eq!(total, 2);

        let (pos, total) = cycling.task_position(&tasks, &tasks[2].id);
        assert_eq!(pos, 2);
        assert_eq!(total, 2);

        // Unknown task
        let unknown_id = TaskId::new();
        let (pos, total) = cycling.task_position(&tasks, &unknown_id);
        assert_eq!(pos, 0);
        assert_eq!(total, 2);
    }
}