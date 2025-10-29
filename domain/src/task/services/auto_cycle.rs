use crate::config::{GeneralConfig, TaskCyclingBehavior};
use crate::task::{Task, Id as TaskId};

/// Pure domain service for auto-cycling logic
///
/// This service contains only pure business logic with no I/O operations.
/// It determines cycling behavior based on configuration and task state.
///
/// # Design Decisions
/// - Pure functions only (no async, no I/O)
/// - Stateless service (all state passed as parameters)
/// - Returns references to avoid unnecessary cloning
///
/// # Evolution Path
/// TODO: Future enhancements:
/// - Priority-based cycling (v2)
/// - Tag-based filtering for cycling (v2)
/// - ML-based task suggestions (v3)
/// - Context-aware cycling based on time of day (v3)
pub struct AutoCycleService;

impl AutoCycleService {
    /// Determines if auto-cycling should be enabled based on configuration
    ///
    /// # Arguments
    /// * `config` - General configuration containing task cycling behavior
    ///
    /// # Returns
    /// * `true` if AutoAdvance or RoundRobin mode is enabled
    /// * `false` if Manual mode is selected
    pub fn should_auto_cycle(config: &GeneralConfig) -> bool {
        matches!(
            config.task_cycling_behavior,
            TaskCyclingBehavior::AutoAdvance | TaskCyclingBehavior::RoundRobin
        )
    }

    /// Selects the next task for auto-cycling from available tasks
    ///
    /// This function implements a simple round-robin cycling through incomplete tasks.
    /// It will skip completed tasks and cycle back to the beginning when reaching the end.
    ///
    /// # Arguments
    /// * `tasks` - Slice of available tasks
    /// * `current_task_id` - Optional ID of the currently active task
    /// * `cycling_behavior` - The configured cycling behavior
    ///
    /// # Returns
    /// * `Some(&Task)` - Reference to the next task to cycle to
    /// * `None` - If no incomplete tasks are available or manual mode is set
    pub fn select_next_task<'a>(
        tasks: &'a [Task],
        current_task_id: Option<&TaskId>,
        cycling_behavior: &TaskCyclingBehavior,
    ) -> Option<&'a Task> {
        match cycling_behavior {
            TaskCyclingBehavior::Manual => None,

            TaskCyclingBehavior::AutoAdvance | TaskCyclingBehavior::RoundRobin => {
                // Filter to incomplete tasks only
                let incomplete_tasks: Vec<&Task> = tasks
                    .iter()
                    .filter(|task| !task.status.is_completed())
                    .collect();

                if incomplete_tasks.is_empty() {
                    return None;
                }

                // Find current task position if provided
                if let Some(current_id) = current_task_id {
                    if let Some(current_pos) = incomplete_tasks
                        .iter()
                        .position(|task| &task.id == current_id)
                    {
                        // Get next task in cycle (wrap around to beginning)
                        let next_pos = (current_pos + 1) % incomplete_tasks.len();
                        return Some(incomplete_tasks[next_pos]);
                    }
                }

                // No current task or not found - return first incomplete task
                incomplete_tasks.first().copied()
            }
        }
    }

    /// Validates if auto-cycling can proceed based on business rules
    ///
    /// # Arguments
    /// * `timer_is_running` - Whether the timer is currently running
    /// * `task_is_completed` - Whether the current task is completed
    ///
    /// # Returns
    /// * `Ok(())` if cycling can proceed
    /// * `Err(reason)` if cycling should be blocked
    pub fn validate_cycling_conditions(
        timer_is_running: bool,
        task_is_completed: bool,
    ) -> Result<(), &'static str> {
        if timer_is_running {
            return Err("Cannot cycle while timer is running");
        }

        if !task_is_completed {
            return Err("Task must be completed before cycling");
        }

        Ok(())
    }

    /// Determines if a task is eligible for cycling to
    ///
    /// # Arguments
    /// * `task` - The task to check
    ///
    /// # Returns
    /// * `true` if the task can be cycled to (not completed)
    /// * `false` if the task should be skipped
    pub fn is_task_eligible(task: &Task) -> bool {
        !task.status.is_completed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Status, Builder as TaskBuilder};

    fn create_test_config(behavior: TaskCyclingBehavior) -> GeneralConfig {
        GeneralConfig {
            task_cycling_behavior: behavior,
            ..Default::default()
        }
    }

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
    fn test_should_auto_cycle() {
        let manual_config = create_test_config(TaskCyclingBehavior::Manual);
        assert!(!AutoCycleService::should_auto_cycle(&manual_config));

        let auto_config = create_test_config(TaskCyclingBehavior::AutoAdvance);
        assert!(AutoCycleService::should_auto_cycle(&auto_config));

        let round_robin_config = create_test_config(TaskCyclingBehavior::RoundRobin);
        assert!(AutoCycleService::should_auto_cycle(&round_robin_config));
    }

    #[test]
    fn test_select_next_task_manual_mode() {
        let tasks = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", false),
        ];

        let result = AutoCycleService::select_next_task(
            &tasks,
            None,
            &TaskCyclingBehavior::Manual,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_select_next_task_auto_advance() {
        let tasks = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", true), // Completed - should be skipped
            create_test_task("Task 3", false),
        ];

        let result = AutoCycleService::select_next_task(
            &tasks,
            Some(&tasks[0].id),
            &TaskCyclingBehavior::AutoAdvance,
        );

        assert_eq!(result.unwrap().name, "Task 3");
    }

    #[test]
    fn test_select_next_task_wrap_around() {
        let tasks = vec![
            create_test_task("Task 1", false),
            create_test_task("Task 2", false),
        ];

        let result = AutoCycleService::select_next_task(
            &tasks,
            Some(&tasks[1].id), // Currently on last task
            &TaskCyclingBehavior::AutoAdvance,
        );

        assert_eq!(result.unwrap().name, "Task 1"); // Wraps to beginning
    }

    #[test]
    fn test_select_next_task_all_completed() {
        let tasks = vec![
            create_test_task("Task 1", true),
            create_test_task("Task 2", true),
        ];

        let result = AutoCycleService::select_next_task(
            &tasks,
            None,
            &TaskCyclingBehavior::AutoAdvance,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_validate_cycling_conditions() {
        assert!(AutoCycleService::validate_cycling_conditions(false, true).is_ok());

        assert_eq!(
            AutoCycleService::validate_cycling_conditions(true, true),
            Err("Cannot cycle while timer is running")
        );

        assert_eq!(
            AutoCycleService::validate_cycling_conditions(false, false),
            Err("Task must be completed before cycling")
        );
    }

    #[test]
    fn test_is_task_eligible() {
        let incomplete_task = create_test_task("Task", false);
        assert!(AutoCycleService::is_task_eligible(&incomplete_task));

        let complete_task = create_test_task("Task", true);
        assert!(!AutoCycleService::is_task_eligible(&complete_task));
    }
}