use domain::{
    Error, EventPublisher, Result, TaskCyclerService, TaskId, TaskRepository,
    TaskSwitchWorkflowCompleted, TimerState, TimerStatus,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchTaskCmd {
    pub task_id: String,
}

pub async fn switch_task(
    timer_state: &mut TimerState,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTaskCmd,
) -> Result<()> {
    if timer_state.status() == TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "TaskSwitch".to_string(),
        });
    }

    let task_id =
        TaskId::from_string(&cmd.task_id).map_err(|_| Error::TaskNotFound {
            id: cmd.task_id.clone(),
        })?;

    let task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: cmd.task_id.clone(),
        }
    })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    cycling_service.validate_task_switch(task_id).await?;

    // Use state transitions to switch task and update configuration
    let result = domain::timer::transitions::StateTransitions::switch_entity(
        timer_state.clone(),
        Some(task_id.to_string()),
    )?;
    *timer_state = result.new_state;

    // Timer configuration update needs to be done through the Timer aggregate
    // For now, we'll just keep the updated state from the switch_task operation
    // The configuration was already applied when we called StateTransitions::switch_entity

    let from_task_id = timer_state
        .active_entity_id()
        .and_then(|id_str| TaskId::from_string(&id_str).ok());
    let switch_event = TaskSwitchWorkflowCompleted::new(
        from_task_id,
        task_id,
        format!("Switched to task: {}", task.name),
        1,
    );
    event_publisher.publish(Box::new(switch_event));

    Ok(())
}

pub async fn switch_to_next_task(
    timer_state: &mut TimerState,
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
) -> Result<Option<String>> {
    if timer_state.status() == TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "NextTaskSwitch".to_string(),
        });
    }

    let current_task_id = timer_state
        .active_entity_id()
        .and_then(|id_str| TaskId::from_string(&id_str).ok());

    let next_task = cycling_service
        .cycle_to_next_active_task(current_task_id)
        .await?;

    if let Some(task) = next_task {
        // Use state transitions to switch task and update configuration
        let result =
            domain::timer::transitions::StateTransitions::switch_entity(
                timer_state.clone(),
                Some(task.id.to_string()),
            )?;
        *timer_state = result.new_state;

        // Then update the configuration
        // Timer configuration update handled during task switch
        // The timer_config has already been incorporated into the state
        Ok(Some(task.id.to_string()))
    } else {
        Ok(None)
    }
}

