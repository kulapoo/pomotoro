use domain::task::Builder as DomainTaskBuilder;
use domain::{Config, Task, TaskId, TaskStatus};

/// Task fixtures for testing
pub struct TaskFixtures;

impl TaskFixtures {
    /// Create a simple task with defaults
    pub fn simple(name: impl Into<String>) -> Task {
        DomainTaskBuilder::with_name_and_sessions(name.into(), 4)
            .build()
            .expect("Failed to create task")
    }

    /// Create a task with custom sessions
    pub fn with_sessions(name: impl Into<String>, sessions: u8) -> Task {
        DomainTaskBuilder::with_name_and_sessions(name.into(), sessions)
            .build()
            .expect("Failed to create task")
    }

    /// Create multiple tasks for testing collections
    pub fn collection(count: usize) -> Vec<Task> {
        (0..count)
            .map(|i| Self::simple(format!("Task {}", i + 1)))
            .collect()
    }

    /// Create the first-boot starter task (same as bootstrap creates).
    ///
    /// This is just a regular task — no special privileges. Provided
    /// here so tests can mimic the post-bootstrap state.
    pub fn starter_task() -> Task {
        DomainTaskBuilder::starter_task()
            .build()
            .expect("Failed to create starter task")
    }

    /// Create a task with starter-task defaults overridden by the
    /// given name and session count. Equivalent to taking the starter
    /// task and renaming/re-sizing it.
    pub fn with_starter_defaults(
        name: impl Into<String>,
        sessions: u8,
    ) -> Task {
        DomainTaskBuilder::starter_task()
            .max_sessions(sessions)
            .name(name.into())
            .build()
            .expect("Failed to create task with starter defaults")
    }
}

/// Builder for creating customized test tasks
/// This wraps the domain Task Builder with test-specific conveniences
pub struct TaskBuilder {
    builder: DomainTaskBuilder,
}

impl TaskBuilder {
    pub fn new() -> Self {
        Self {
            builder: DomainTaskBuilder::new(),
        }
    }

    /// Start from the first-boot starter task configuration.
    pub fn starter() -> Self {
        Self {
            builder: DomainTaskBuilder::starter_task(),
        }
    }

    pub fn id(mut self, id: TaskId) -> Self {
        self.builder = self.builder.id(id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.builder = self.builder.name(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.builder = self.builder.description(desc.into());
        self
    }

    pub fn max_sessions(mut self, sessions: u8) -> Self {
        self.builder = self.builder.max_sessions(sessions);
        self
    }

    pub fn current_sessions(mut self, sessions: u8) -> Self {
        self.builder = self.builder.current_sessions(sessions);
        self
    }

    pub fn status(mut self, status: TaskStatus) -> Self {
        self.builder = self.builder.status(status);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.builder = self.builder.tags(tags);
        self
    }

    pub fn as_active(mut self) -> Self {
        self.builder = self.builder.status(TaskStatus::Active);
        self
    }

    pub fn as_completed(mut self) -> Self {
        self.builder = self
            .builder
            .status(TaskStatus::Completed)
            .current_sessions(4)
            .max_sessions(4);
        self
    }

    pub fn config(mut self, config: Config) -> Self {
        self.builder = self.builder.config(config);
        self
    }

    pub fn build(self) -> Task {
        self.builder.build().expect("Failed to build task")
    }
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_simple_task() {
        let task = TaskFixtures::simple("Test Task");
        assert_eq!(task.name(), "Test Task");
        assert_eq!(task.max_sessions(), 4);
    }

    #[test]
    fn creates_task_with_builder() {
        let task = TaskBuilder::new()
            .name("Custom Task")
            .max_sessions(6)
            .as_active()
            .build();

        assert_eq!(task.name(), "Custom Task");
        assert_eq!(task.max_sessions(), 6);
        assert_eq!(task.status(), TaskStatus::Active);
    }

    #[test]
    fn creates_task_collection() {
        let tasks = TaskFixtures::collection(3);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].name(), "Task 1");
        assert_eq!(tasks[2].name(), "Task 3");
    }
}
