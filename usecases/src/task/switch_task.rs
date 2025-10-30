use domain::{
    Error, EventPublisher, Result, Task, TaskId, TaskRepository, TaskSwitchWorkflowCompleted, Timer, TimerRepository
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchTaskCmd {
    pub task_id: TaskId,
}

/// Pure validation function for task switching
pub fn validate_task_switch(
    task: &Task,
    timer: &Timer
) -> Result<()> {
    // Check if task is completed
    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Check if timer is running
    if timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "TaskSwitch".to_string(),
        });
    }

    Ok(())
}

/// Switch to a different task (requires async for state management)
/// This function coordinates the task switching workflow
pub async fn switch_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTaskCmd,
) -> Result<()> {
    // Get the target task
    let task = task_repo.get_by_id(cmd.task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: cmd.task_id.to_string(),
        }
    })?;

    // Get the single timer instance
    let mut timer = timer_repo.get().await?;

    // Use pure validation
    validate_task_switch(&task, &timer)?;

    // Update the timer's active task
    let previous_task_id = timer.active_task_id();
    timer.set_active_task(cmd.task_id);
    timer_repo.save(&timer).await?;

    // Publish event
    let switch_event = TaskSwitchWorkflowCompleted::new(
        previous_task_id,
        cmd.task_id,
        format!("Switched to task: {}", task.name),
        1,
    );
    event_publisher.publish(Box::new(switch_event));

    Ok(())
}