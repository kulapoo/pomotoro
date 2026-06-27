use domain::{
    Error, EventPublisher, Result, Task, TaskActiveChanged, TaskId,
    TaskRepository, TaskStatus, TaskUpdated, Timer, TimerRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchTaskCmd {
    pub task_id: TaskId,
}

/// Pure validation function for task switching
pub fn validate_task_switch(task: &Task, timer: &Timer) -> Result<()> {
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
///
/// ## Business Rules
/// - Task must exist and not be completed
/// - Updates task statuses: previous task → Queued, new task → Active
/// - Updates timer's active task reference
/// - Publishes appropriate domain events
/// - By default, prevents switching while timer is running (use switch_task_force for UI/bootstrap)
pub async fn switch_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTaskCmd,
) -> Result<()> {
    // Get the target task
    let mut task =
        task_repo.get_by_id(cmd.task_id).await?.ok_or_else(|| {
            Error::TaskNotFound {
                id: cmd.task_id.to_string(),
            }
        })?;

    // Get the single timer instance
    let timer = timer_repo.get().await?;

    // Use pure validation - check if timer is running
    validate_task_switch(&task, &timer)?;

    // Handle previous active task status transition
    let previous_task_id = timer.task_id();
    if let Some(prev_task_id) = previous_task_id {
        if prev_task_id != cmd.task_id {
            if let Some(mut prev_task) =
                task_repo.get_by_id(prev_task_id).await?
            {
                if prev_task.status() != TaskStatus::Completed {
                    prev_task.queue()?;
                    let prev_id = prev_task.id();
                    task_repo.update(prev_task).await?;

                    // Publish TaskUpdated event for previous task
                    event_publisher.publish(Box::new(TaskUpdated::new(
                        prev_id, None, None, None, None, 1,
                    )));
                }
            }
        }
    }

    // Set new task status to Active
    task.activate()?;
    let task_name = task.name().to_string();
    let task_id = task.id();
    task_repo.update(task).await?;

    // Bind the new task in the Idle state. The previous task's phase/remaining
    // is NOT carried over — invariant matching `switch_active_task`.
    let new_timer = domain::Timer::new(cmd.task_id);
    timer_repo.save(&new_timer).await?;

    // Publish TaskUpdated event for new active task — after timer save so
    // listeners see the fully consistent state (timer + task together).
    event_publisher.publish(Box::new(TaskUpdated::new(
        task_id, None, None, None, None, 1,
    )));

    // Publish TaskActiveChanged event
    let switch_event = TaskActiveChanged::new(
        previous_task_id,
        cmd.task_id,
        format!("Switched to task: {}", task_name),
        1,
    );
    event_publisher.publish(Box::new(switch_event));

    Ok(())
}
