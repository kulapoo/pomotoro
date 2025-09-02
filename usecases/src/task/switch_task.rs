use domain::{
    Error, EventPublisher, Result, TaskCyclerService, TaskId, TaskRepository,
    TaskSwitchWorkflowCompleted, TimerRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchTaskCmd {
    pub task_id: String,
}

/// Switch to a different task's timer
/// In the new architecture, each task has its own timer,
/// so switching tasks means switching which timer is active.
pub async fn switch_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTaskCmd,
) -> Result<()> {
    let task_id =
        TaskId::from_string(&cmd.task_id).map_err(|_| Error::TaskNotFound {
            id: cmd.task_id.clone(),
        })?;

    // Get the target task
    let task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: cmd.task_id.clone(),
        }
    })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Check if the task's timer is already running
    let timer = timer_repo
        .get_by_id(task.get_timer_id())
        .await?
        .ok_or_else(|| Error::RepositoryError {
            message: format!("Timer not found for task: {}", cmd.task_id),
        })?;

    if timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "TaskSwitch".to_string(),
        });
    }

    cycling_service.validate_task_switch(task_id).await?;

    let switch_event = TaskSwitchWorkflowCompleted::new(
        None, // No from_task_id in new architecture
        task_id,
        format!("Switched to task: {}", task.name),
        1,
    );
    event_publisher.publish(Box::new(switch_event));

    Ok(())
}

pub async fn switch_to_next_task(
    current_task_id: Option<TaskId>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
) -> Result<Option<String>> {
    // If there's a current task, check its timer isn't running
    if let Some(task_id) = current_task_id {
        if let Some(task) = task_repo.get_by_id(task_id).await? {
            if let Some(timer) =
                timer_repo.get_by_id(task.get_timer_id()).await?
            {
                if timer.is_running() {
                    return Err(Error::InvalidStateTransition {
                        from: "Running".to_string(),
                        to: "NextTaskSwitch".to_string(),
                    });
                }
            }
        }
    }

    let next_task = cycling_service
        .cycle_to_next_active_task(current_task_id)
        .await?;

    if let Some(task) = next_task {
        Ok(Some(task.id.to_string()))
    } else {
        Ok(None)
    }
}
