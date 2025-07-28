#[cfg(test)]
mod tests {
    use crate::{Task, TaskBuilder, TaskDefaults, TaskStatus};
    use std::time::Duration;

    fn default_config() -> TaskDefaults {
        TaskDefaults {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
            enable_screen_blocking: false,
            max_sessions_default: 4,
        }
    }

    #[test]
    fn test_basic_task_creation() {
        let defaults = default_config();
        let task = TaskBuilder::with_name_and_sessions("Test Task".to_string(), 3)
            .build_with_defaults(&defaults)
            .unwrap();

        assert_eq!(task.name, "Test Task");
        assert_eq!(task.max_sessions, 3);
        assert_eq!(task.current_sessions, 0);
        assert_eq!(task.status, TaskStatus::Queued);
        assert!(task.description.is_none());
        assert!(task.tags.is_empty());
    }

    #[test]
    fn test_fluent_builder_interface() {
        let defaults = default_config();
        let task = TaskBuilder::new()
            .name("Fluent Task".to_string())
            .max_sessions(2)
            .description("A task built with fluent interface".to_string())
            .tags(vec!["work".to_string(), "urgent".to_string()])
            .status(TaskStatus::Active)
            .build_with_defaults(&defaults)
            .unwrap();

        assert_eq!(task.name, "Fluent Task");
        assert_eq!(task.max_sessions, 2);
        assert_eq!(task.description, Some("A task built with fluent interface".to_string()));
        assert_eq!(task.tags, vec!["work", "urgent"]);
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn test_default_task_creation() {
        let defaults = default_config();
        let task = TaskBuilder::default_task()
            .build_with_defaults(&defaults)
            .unwrap();

        assert_eq!(task.name, "Focus Session");
        assert_eq!(task.max_sessions, defaults.max_sessions_default);
        assert_eq!(task.description, Some("Default pomodoro task for focused work".to_string()));
        assert_eq!(task.tags, vec!["focus"]);
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn test_validation_empty_name() {
        let defaults = default_config();
        let result = TaskBuilder::with_name_and_sessions("".to_string(), 3)
            .build_with_defaults(&defaults);

        assert!(result.is_err());
    }

    #[test]
    fn test_validation_zero_max_sessions() {
        let defaults = default_config();
        let result = TaskBuilder::with_name_and_sessions("Test".to_string(), 0)
            .build_with_defaults(&defaults);

        assert!(result.is_err());
    }

    #[test]
    fn test_completed_task() {
        let defaults = default_config();
        let task = TaskBuilder::with_name_and_sessions("Completed Task".to_string(), 2)
            .current_sessions(2)
            .completed()
            .build_with_defaults(&defaults)
            .unwrap();

        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        assert_eq!(task.current_sessions, 2);
    }

    #[test]
    fn test_backward_compatibility() {
        let defaults = default_config();
        
        // Test that existing constructors still work
        let task1 = Task::new_with_defaults("Test Task".to_string(), 3, &defaults).unwrap();
        let task2 = Task::new_default().unwrap();

        assert_eq!(task1.name, "Test Task");
        assert_eq!(task1.max_sessions, 3);
        
        assert_eq!(task2.name, "Focus Session");
        assert_eq!(task2.max_sessions, 4); // Built-in default
    }
}