use crate::config::{GeneralConfig, TaskCyclingBehavior};
use crate::task::{Id as TaskId, Task};

/// Pure domain service for auto-cycling logic.
///
/// Contains only business logic with no I/O operations.
pub struct CycleService;

impl CycleService {
    /// Returns true if auto-cycling should be enabled based on configuration.
    pub fn should_auto_cycle(config: &GeneralConfig) -> bool {
        matches!(
            config.task_cycling_behavior,
            TaskCyclingBehavior::AutoAdvance | TaskCyclingBehavior::RoundRobin
        )
    }

    /// Selects the next task for auto-cycling using round-robin strategy.
    ///
    /// Returns `None` if:
    /// - Manual mode is enabled
    /// - No incomplete tasks are available
    /// - Current task not found in incomplete tasks (with no current task, returns first)
    pub fn select_next_task<'a>(
        tasks: &'a [Task],
        current_task_id: Option<&TaskId>,
        cycling_behavior: &TaskCyclingBehavior,
    ) -> Option<&'a Task> {
        match cycling_behavior {
            TaskCyclingBehavior::Manual => None,
            TaskCyclingBehavior::AutoAdvance
            | TaskCyclingBehavior::RoundRobin => {
                Self::next_round_robin_task(tasks, current_task_id)
            }
        }
    }

    /// Validates if auto-cycling can proceed based on business rules.
    ///
    /// Returns `Ok(())` if cycling can proceed, `Err(reason)` if blocked.
    pub fn validate_cycling_conditions(
        timer_is_running: bool,
        task_is_completed: bool,
    ) -> Result<(), &'static str> {
        if timer_is_running {
            Err("Cannot cycle while timer is running")
        } else if !task_is_completed {
            Err("Task must be completed before cycling")
        } else {
            Ok(())
        }
    }

    /// Returns true if the task can be cycled to (not completed).
    pub fn is_task_eligible(task: &Task) -> bool {
        !task.status.is_completed()
    }

    /// Round-robin implementation that cycles through incomplete tasks.
    fn next_round_robin_task<'a>(
        tasks: &'a [Task],
        current_task_id: Option<&TaskId>,
    ) -> Option<&'a Task> {
        let incomplete_tasks: Vec<&Task> = tasks
            .iter()
            .filter(|task| Self::is_task_eligible(task))
            .collect();

        if incomplete_tasks.is_empty() {
            return None;
        }

        let current_pos = current_task_id.and_then(|id| {
            incomplete_tasks.iter().position(|task| &task.id == id)
        });

        match current_pos {
            Some(pos) => {
                let next_pos = (pos + 1) % incomplete_tasks.len();
                Some(incomplete_tasks[next_pos])
            }
            None => Some(incomplete_tasks[0]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Builder as TaskBuilder, Status};

    fn create_config(behavior: TaskCyclingBehavior) -> GeneralConfig {
        GeneralConfig {
            task_cycling_behavior: behavior,
            ..Default::default()
        }
    }

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
    fn test_should_auto_cycle() {
        assert!(!CycleService::should_auto_cycle(&create_config(
            TaskCyclingBehavior::Manual
        )));
        assert!(CycleService::should_auto_cycle(&create_config(
            TaskCyclingBehavior::AutoAdvance
        )));
        assert!(CycleService::should_auto_cycle(&create_config(
            TaskCyclingBehavior::RoundRobin
        )));
    }

    #[test]
    fn test_select_next_task_manual_mode() {
        let tasks =
            vec![create_task("Task 1", false), create_task("Task 2", false)];

        let result = CycleService::select_next_task(
            &tasks,
            None,
            &TaskCyclingBehavior::Manual,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_select_next_task_skips_completed() {
        let tasks = vec![
            create_task("Task 1", false),
            create_task("Task 2", true), // Completed - skipped
            create_task("Task 3", false),
        ];

        let result = CycleService::select_next_task(
            &tasks,
            Some(&tasks[0].id),
            &TaskCyclingBehavior::AutoAdvance,
        )
        .unwrap();

        assert_eq!(result.name, "Task 3");
    }

    #[test]
    fn test_select_next_task_wrap_around() {
        let tasks =
            vec![create_task("Task 1", false), create_task("Task 2", false)];

        let result = CycleService::select_next_task(
            &tasks,
            Some(&tasks[1].id), // Last task
            &TaskCyclingBehavior::AutoAdvance,
        )
        .unwrap();

        assert_eq!(result.name, "Task 1"); // Wraps to beginning
    }

    #[test]
    fn test_select_next_task_no_current_returns_first() {
        let tasks =
            vec![create_task("Task 1", false), create_task("Task 2", false)];

        let result = CycleService::select_next_task(
            &tasks,
            None,
            &TaskCyclingBehavior::AutoAdvance,
        )
        .unwrap();

        assert_eq!(result.name, "Task 1");
    }

    #[test]
    fn test_validate_cycling_conditions() {
        assert!(CycleService::validate_cycling_conditions(false, true).is_ok());
        assert!(CycleService::validate_cycling_conditions(true, true).is_err());
        assert!(
            CycleService::validate_cycling_conditions(false, false).is_err()
        );
    }

    #[test]
    fn test_is_task_eligible() {
        assert!(CycleService::is_task_eligible(&create_task("Task", false)));
        assert!(!CycleService::is_task_eligible(&create_task("Task", true)));
    }
}
