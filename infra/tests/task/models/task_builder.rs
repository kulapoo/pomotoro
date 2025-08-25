use domain::{Task, TaskStatus};

pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    pub fn new(name: String, max_sessions: u8) -> Self {
        let task = Task::new(name, max_sessions)
            .expect("Failed to create task in test");
        Self { task }
    }

    pub fn with_description(mut self, description: String) -> Self {
        // Task doesn't have with_description, need to modify field directly
        self.task.description = Some(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        // Task doesn't have with_tags, need to modify field directly
        self.task.tags = tags;
        self
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.task.status = status;
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
