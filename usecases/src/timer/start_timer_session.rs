use domain::timer::TimerService;
use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, timer::Started,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StartTimerSessionCmd {
    pub task_id: Option<String>,
}

/// Start a timer session with optional task switching
///
/// This use case orchestrates starting a pomodoro timer session,
/// optionally switching to a specific task first. It coordinates
/// between the task repository and timer service to ensure proper
/// business logic execution.
///
/// ## Business Rules
///
/// - Task must exist and not be completed if task_id is provided
/// - Timer must not already be running
/// - An active task must be available (either provided or existing)
///
/// ## Dependencies
///
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For task validation and retrieval
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn start_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: StartTimerSessionCmd,
) -> Result<()> {
    let task = if let Some(task_id_str) = cmd.task_id {
        let task_id = TaskId::from_string(&task_id_str).map_err(|_| {
            Error::TaskNotFound {
                id: task_id_str.clone(),
            }
        })?;

        let task = task_repo
            .get_by_id(task_id)
            .await?
            .ok_or(Error::TaskNotFound { id: task_id_str })?;

        if task.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }

        timer_service.switch_task(task_id, Some(&task)).await?;

        Some(task)
    } else {
        let current_state = timer_service.get_state().await?;

        if current_state.active_entity_id().is_none() {
            return Err(Error::InvalidStateTransition {
                from: "no_active_task".to_string(),
                to: "start_session".to_string(),
            });
        }

        if let Some(entity_id_str) = current_state.active_entity_id() {
            if let Ok(task_id) = TaskId::from_string(&entity_id_str) {
                task_repo.get_by_id(task_id).await?
            } else {
                None
            }
        } else {
            None
        }
    };

    timer_service.start_timer(task.as_ref()).await?;

    let updated_state = timer_service.get_state().await?;
    let timer_started_event = Started::new(
        updated_state.active_entity_id(),
        updated_state.phase(),
        updated_state.remaining_seconds(),
        1, // version
    );
    event_publisher.publish(Box::new(timer_started_event));

    Ok(())
}
