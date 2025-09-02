use domain::{
    Config, EventPublisher, Result, Task, TaskBuilder, TaskCreated, TaskRepository,
    TimerConfiguration,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CreateTaskCmd {
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
}

pub async fn create_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: CreateTaskCmd,
) -> Result<Task> {
    // Build the task with default timer configuration
    let timer_config = TimerConfiguration::default();
    let config = Config {
        timer: timer_config,
        ..Default::default()
    };
    let mut builder =
        TaskBuilder::with_name_and_sessions(cmd.name, cmd.max_sessions)
            .config(config.clone());

    if let Some(description) = cmd.description {
        builder = builder.with_description(description);
    }

    if !cmd.tags.is_empty() {
        builder = builder.with_tags(cmd.tags);
    }

    let task = builder.build()?;

    // Create the task
    task_repo.create(task.clone()).await?;

    let created_event = TaskCreated::new(
        task.id,
        task.name.clone(),
        task.description.clone(),
        task.max_sessions,
        task.tags.clone(),
        config,
        1,
    );
    event_publisher.publish(Box::new(created_event));

    Ok(task)
}
