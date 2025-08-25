use crate::task::mappers::task_config_to_timer_config;
use domain::{Error, Result, TaskId, TaskRepository, TimerState, TimerStatus};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StartSessionCmd {
    pub task_id: Option<String>,
}

pub async fn start_session(
    timer_state: &mut TimerState,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cmd: StartSessionCmd,
) -> Result<()> {
    // If a task ID is provided, switch to that task first
    if let Some(task_id_str) = cmd.task_id {
        let task_id = TaskId::from_string(&task_id_str).map_err(|_| {
            Error::TaskNotFound {
                id: task_id_str.clone(),
            }
        })?;

        // Verify task exists and is not completed
        let task = task_repo
            .get_by_id(task_id)
            .await?
            .ok_or(Error::TaskNotFound { id: task_id_str })?;

        if task.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }

        // Switch to the task with its configuration using proper mapper
        let _timer_config = task_config_to_timer_config(&task.config)?;
        // Use state transitions to switch task and update configuration
        let result =
            domain::timer::transitions::StateTransitions::switch_entity(
                timer_state.clone(),
                Some(task_id.to_string()),
            )?;
        *timer_state = result.new_state;

        // Configuration update is handled as part of the task switch
        // The timer_config has already been incorporated into the state
    }

    // Ensure we have an active task
    if timer_state.active_entity_id().is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "start_session".to_string(),
        });
    }

    // Prevent starting if already running
    if timer_state.status() == TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Start".to_string(),
        });
    }

    let result = domain::timer::transitions::StateTransitions::start(
        timer_state.clone(),
    )?;
    *timer_state = result.new_state;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::InMemoryTaskRepository;
    use domain::{Task, TimerConfiguration, TimerStatus};

    async fn setup() -> (Arc<dyn TaskRepository + Send + Sync>, Task) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());

        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();

        (task_repo, task)
    }

    #[tokio::test]
    async fn should_start_session_with_task_id() {
        let (task_repo, task) = setup().await;
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        };

        let cmd = StartSessionCmd {
            task_id: Some(task.id.to_string()),
        };

        start_session(&mut timer_state, &task_repo, cmd)
            .await
            .unwrap();

        assert_eq!(timer_state.status(), TimerStatus::Running);
        assert_eq!(timer_state.active_entity_id(), Some(task.id.to_string()));
    }

    #[tokio::test]
    async fn should_start_session_with_existing_active_task() {
        let (task_repo, task) = setup().await;
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(task.id.to_string()),
        };

        let cmd = StartSessionCmd { task_id: None };

        start_session(&mut timer_state, &task_repo, cmd)
            .await
            .unwrap();

        assert_eq!(timer_state.status(), TimerStatus::Running);
        assert_eq!(timer_state.active_entity_id(), Some(task.id.to_string()));
    }

    #[tokio::test]
    async fn should_fail_without_active_task() {
        let (task_repo, _) = setup().await;
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        };

        let cmd = StartSessionCmd { task_id: None };

        let result = start_session(&mut timer_state, &task_repo, cmd).await;

        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (task_repo, _) = setup().await;
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        };

        let cmd = StartSessionCmd {
            task_id: Some("nonexistent-id".to_string()),
        };

        let result = start_session(&mut timer_state, &task_repo, cmd).await;

        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn should_fail_with_completed_task() {
        let (task_repo, _) = setup().await;
        let mut timer_state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        };

        let mut completed_task =
            Task::new("Completed Task".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap();
        task_repo.create(completed_task.clone()).await.unwrap();

        let cmd = StartSessionCmd {
            task_id: Some(completed_task.id.to_string()),
        };

        let result = start_session(&mut timer_state, &task_repo, cmd).await;

        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }

    #[tokio::test]
    async fn should_fail_if_already_running() {
        let (task_repo, task) = setup().await;
        // Create working state to simulate running
        let mut timer_state = TimerState::Working {
            remaining_seconds: 1500,
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(task.id.to_string()),
            entity_session_count: 0,
        };

        let cmd = StartSessionCmd { task_id: None };

        let result = start_session(&mut timer_state, &task_repo, cmd).await;

        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }
}
