use crate::task::complete_session;
use domain::{
    Error, EventPublisher, Phase, Result, TaskRepository, TimerState,
    timer::PhaseCompleted,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SessionCompleted {
    pub old_phase: Phase,
    pub new_phase: Phase,
    pub work_session_completed: bool,
    pub cycle_completed: bool,
    pub task_completed: bool,
    pub sessions_completed: u8,
    pub total_sessions: u8,
}

pub async fn complete_timer_session(
    timer_state: &mut TimerState,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<SessionCompleted> {
    let active_task_id = timer_state.active_entity_id().ok_or_else(|| {
        Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "complete_session".to_string(),
        }
    })?;

    let old_phase = timer_state.phase();

    // Complete the task session if this was a work phase
    let task_completed = if old_phase == Phase::Work {
        let task_id_str = active_task_id.to_string();
        let result = complete_session::complete_session(
            task_repo,
            event_publisher,
            &task_id_str,
        )
        .await?;
        result.task_completed
    } else {
        false
    };

    // TODO: Implement phase transition logic using new state machine
    let new_phase = old_phase; // Placeholder

    // Prepare session completion info
    let result = SessionCompleted {
        old_phase,
        new_phase,
        work_session_completed: old_phase == Phase::Work,
        cycle_completed: false, // TODO: Implement cycle detection
        task_completed,
        sessions_completed: timer_state.session_count() as u8,
        total_sessions: timer_state.configuration().sessions_until_long_break,
    };

    // Publish phase completed event
    let event = PhaseCompleted::new(
        timer_state.active_entity_id(),
        result.old_phase,
        result.new_phase,
        result.sessions_completed as u32,
        0, // task_session_count - TODO: track this in timer state
        1, // version - TODO: implement proper versioning
    );
    event_publisher.publish(Box::new(event));

    Ok(result)
}

pub async fn handle_phase_completion(
    timer_state: &mut TimerState,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let _result =
        complete_timer_session(timer_state, task_repo, event_publisher).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        InMemoryTaskRepository, NoOpEventPublisher, Task, TaskId,
        TimerConfiguration,
    };

    async fn setup() -> (
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
        TaskId,
    ) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(NoOpEventPublisher);

        // Create and save a test task
        let task = Task::new("Test Task".to_string(), 3).unwrap();
        let task_id = task.id;
        task_repo.create(task).await.unwrap();

        (task_repo, event_publisher, task_id)
    }

    #[tokio::test]
    async fn should_complete_work_session() {
        let (task_repo, event_publisher, task_id) = setup().await;

        let mut timer_state = TimerState::Working {
            remaining_seconds: 0,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: Some(task_id.to_string()),
            entity_session_count: 0,
        };

        let result = complete_timer_session(
            &mut timer_state,
            &task_repo,
            &event_publisher,
        )
        .await
        .unwrap();

        assert!(result.work_session_completed);
        assert_eq!(result.old_phase, Phase::Work);
    }

    #[tokio::test]
    async fn should_fail_without_active_task() {
        let (task_repo, event_publisher, _) = setup().await;

        let mut timer_state = TimerState::Working {
            remaining_seconds: 0,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: None,
            entity_session_count: 0,
        };

        let result = complete_timer_session(
            &mut timer_state,
            &task_repo,
            &event_publisher,
        )
        .await;

        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }
}
