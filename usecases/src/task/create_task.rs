use domain::{
    Config, EventPublisher, Result, Task, TaskBuilder, TaskCreated, TaskRepository,
    Timer, TimerConfiguration, TimerId, TimerRepository,
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
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: CreateTaskCmd,
) -> Result<Task> {
    // Create a timer for this task first
    let timer_id = TimerId::new();
    let timer_config = TimerConfiguration::default();
    let timer = Timer::new(timer_id, timer_config.clone());
    timer_repo.create(timer).await?;

    // Build the task with the timer_id and Config
    let config = Config {
        timer: timer_config,
        ..Default::default()
    };
    let mut builder =
        TaskBuilder::with_name_and_sessions(cmd.name, cmd.max_sessions)
            .timer_id(timer_id)
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
