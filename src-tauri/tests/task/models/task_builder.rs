use pomotoro_lib::task::models::{Task, TaskStatus};
use std::time::Duration;
use uuid::Uuid;

pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    pub fn new(name: String, max_sessions: u8) -> Self {
        let task = Task::new(name, max_sessions).expect("Failed to create task in test");
        Self { task }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.task.id = id;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.task = self.task.with_description(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.task = self.task.with_tags(tags);
        self
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.task.status = status;
        self
    }

    pub fn with_sessions(mut self, current: u8) -> Self {
        for _ in 0..current {
            let _ = self.task.increment_session();
        }
        self
    }

    pub fn with_work_duration(mut self, duration: Duration) -> Self {
        self.task.config.work_duration = duration;
        self
    }

    pub fn with_short_break_duration(mut self, duration: Duration) -> Self {
        self.task.config.short_break_duration = duration;
        self
    }

    pub fn with_long_break_duration(mut self, duration: Duration) -> Self {
        self.task.config.long_break_duration = duration;
        self
    }

    pub fn completed(mut self) -> Self {
        self.task.status = TaskStatus::Completed;
        self.task.current_sessions = self.task.max_sessions;
        self
    }

    pub fn build(self) -> Task {
        self.task
    }
}