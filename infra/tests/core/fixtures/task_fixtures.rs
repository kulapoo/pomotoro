use domain::{Task, TaskId, TaskStatus};
use domain::task::Builder as DomainTaskBuilder;

/// Task fixtures for testing
pub struct TaskFixtures;

impl TaskFixtures {
    /// Create a simple task with defaults
    pub fn simple(name: impl Into<String>) -> Task {
        Task::new(name.into(), 4).expect("Failed to create task")
    }

    /// Create a task with custom sessions
    pub fn with_sessions(name: impl Into<String>, sessions: u8) -> Task {
        Task::new(name.into(), sessions).expect("Failed to create task")
    }

    /// Create multiple tasks for testing collections
    pub fn collection(count: usize) -> Vec<Task> {
        (0..count)
            .map(|i| Self::simple(format!("Task {}", i + 1)))
            .collect()
    }

    /// Create a default task (the one marked as default in the system)
    pub fn default_task() -> Task {
        Task::new_default().expect("Failed to create default task")
    }

    /// Create a task with default settings
    pub fn with_defaults(name: impl Into<String>) -> Task {
        Task::new_with_defaults(name.into(), 4)
            .expect("Failed to create task with defaults")
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

    pub fn default() -> Self {
        Self {
            builder: DomainTaskBuilder::default_task(),
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

    pub fn as_default(mut self) -> Self {
        self.builder = self.builder.default(true);
        self
    }

    pub fn as_active(mut self) -> Self {
        self.builder = self.builder.status(TaskStatus::Active);
        self
    }

    pub fn as_completed(mut self) -> Self {
        self.builder = self.builder
            .status(TaskStatus::Completed)
            .current_sessions(4)
            .max_sessions(4);
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
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.max_sessions, 4);
    }

    #[test]
    fn creates_task_with_builder() {
        let task = TaskBuilder::new()
            .name("Custom Task")
            .max_sessions(6)
            .as_active()
            .build();

        assert_eq!(task.name, "Custom Task");
        assert_eq!(task.max_sessions, 6);
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn creates_task_collection() {
        let tasks = TaskFixtures::collection(3);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].name, "Task 1");
        assert_eq!(tasks[2].name, "Task 3");
    }
}