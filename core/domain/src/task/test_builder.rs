#[cfg(test)]
mod tests {
    use crate::{Task, TaskBuilder, TaskStatus};

    #[test]
    fn test_basic_task_creation() {
        let task =
            TaskBuilder::with_name_and_sessions("Test Task".to_string(), 3)
                .build()
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
        let task = TaskBuilder::new()
            .name("Fluent Task".to_string())
            .max_sessions(2)
            .description("A task built with fluent interface".to_string())
            .tags(vec!["work".to_string(), "urgent".to_string()])
            .status(TaskStatus::Active)
            .build()
            .unwrap();

        assert_eq!(task.name, "Fluent Task");
        assert_eq!(task.max_sessions, 2);
        assert_eq!(
            task.description,
            Some("A task built with fluent interface".to_string())
        );
        assert_eq!(task.tags, vec!["work", "urgent"]);
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn test_starter_task_creation() {
        let task = TaskBuilder::starter_task().build().unwrap();

        assert_eq!(task.name, "Focus Session");
        assert_eq!(task.max_sessions, 4);
        assert_eq!(
            task.description,
            Some("Default pomodoro task for focused work".to_string())
        );
        assert_eq!(task.tags, vec!["focus"]);
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn test_validation_empty_name() {
        let result =
            TaskBuilder::with_name_and_sessions("".to_string(), 3).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_validation_zero_max_sessions() {
        let result =
            TaskBuilder::with_name_and_sessions("Test".to_string(), 0).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_completed_task() {
        let task = TaskBuilder::with_name_and_sessions(
            "Completed Task".to_string(),
            2,
        )
        .current_sessions(2)
        .completed()
        .build()
        .unwrap();

        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        assert_eq!(task.current_sessions, 2);
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that existing constructors still work
        let task1 =
            Task::new_with_defaults("Test Task".to_string(), 3).unwrap();
        let task2 = TaskBuilder::starter_task().build().unwrap();

        assert_eq!(task1.name, "Test Task");
        assert_eq!(task1.max_sessions, 3);

        assert_eq!(task2.name, "Focus Session");
        assert_eq!(task2.max_sessions, 4); // Built-in default
    }

    #[test]
    fn test_starter_task_has_expected_fields() {
        let task = TaskBuilder::starter_task().build().unwrap();

        assert_eq!(task.name, "Focus Session");
        assert_eq!(
            task.description,
            Some("Default pomodoro task for focused work".to_string())
        );
        assert_eq!(task.tags, vec!["focus"]);
        assert_eq!(task.status, TaskStatus::Active);
    }
}
