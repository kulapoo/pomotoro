use domain::timer::TimerService;
use domain::{Error, EventPublisher, Result, TaskId, TaskRepository, TimerStarted};
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
        let task_id = TaskId::from_string(&task_id_str).map_err(|_| Error::TaskNotFound {
            id: task_id_str.clone(),
        })?;

        let task = task_repo
            .get_by_id(task_id.clone())
            .await?
            .ok_or_else(|| Error::TaskNotFound { id: task_id_str })?;

        if task.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }

        timer_service.switch_task(task_id, Some(&task)).await?;

        Some(task)
    } else {
        let current_state = timer_service.get_state().await?;
        if current_state.active_task_id.is_none() {
            return Err(Error::InvalidStateTransition {
                from: "no_active_task".to_string(),
                to: "start_session".to_string(),
            });
        }

        if let Some(task_id) = current_state.active_task_id {
            task_repo.get_by_id(task_id).await?
        } else {
            None
        }
    };

    timer_service.start_timer(task.as_ref()).await?;

    let updated_state = timer_service.get_state().await?;
    let timer_started_event = TimerStarted::new(
        updated_state.active_task_id.clone(),
        updated_state.timer.phase,
        updated_state.timer.remaining_seconds,
        1, // version
    );
    event_publisher.publish(Box::new(timer_started_event));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::InMemoryTaskRepository;
    use domain::{Task, TimerState, TimerStatus};
    use std::sync::{Arc, RwLock};

    struct MockTimerService {
        state: Arc<RwLock<TimerState>>,
    }

    impl MockTimerService {
        fn new() -> Self {
            Self {
                state: Arc::new(RwLock::new(TimerState::default())),
            }
        }
    }

    #[async_trait]
    impl TimerService for MockTimerService {
        async fn start_timer(&self, _task: Option<&Task>) -> Result<()> {
            let mut state = self.state.write().unwrap();
            state.set_status(TimerStatus::Running)?;
            Ok(())
        }

        async fn stop_timer(&self) -> Result<()> {
            let mut state = self.state.write().unwrap();
            state.set_status(TimerStatus::Stopped)?;
            Ok(())
        }

        async fn toggle_pause(&self) -> Result<TimerStatus> {
            let mut state = self.state.write().unwrap();
            let new_status = match state.status() {
                TimerStatus::Running => TimerStatus::Paused,
                TimerStatus::Paused => TimerStatus::Running,
                TimerStatus::Stopped => TimerStatus::Stopped,
            };
            state.set_status(new_status)?;
            Ok(new_status)
        }

        async fn reset_current_phase(&self, _task: Option<&Task>) -> Result<()> {
            let mut state = self.state.write().unwrap();
            state.reset_current_phase();
            Ok(())
        }

        async fn skip_to_next_phase(
            &self,
            _task: Option<&Task>,
        ) -> Result<(domain::Phase, domain::Phase)> {
            let mut state = self.state.write().unwrap();
            state.next_phase()
        }

        async fn get_state(&self) -> Result<TimerState> {
            Ok(self.state.read().unwrap().clone())
        }

        async fn switch_task(&self, task_id: TaskId, _task: Option<&Task>) -> Result<()> {
            let mut state = self.state.write().unwrap();
            state.switch_task(task_id)?;
            Ok(())
        }

        async fn load_state(&self) -> Result<()> {
            Ok(())
        }

        async fn save_state(&self) -> Result<()> {
            Ok(())
        }
    }

    async fn setup() -> (
        Arc<dyn TimerService + Send + Sync>,
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
        Task,
    ) {
        let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(domain::NoOpEventPublisher);

        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();

        (timer_service, task_repo, event_publisher, task)
    }

    #[tokio::test]
    async fn should_start_timer_session_with_task_id() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;

        let cmd = StartTimerSessionCmd {
            task_id: Some(task.id.to_string()),
        };

        start_timer_session(&timer_service, &task_repo, &event_publisher, cmd)
            .await
            .unwrap();

        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Running);
        assert_eq!(state.active_task_id, Some(task.id));
    }

    #[tokio::test]
    async fn should_start_timer_session_with_existing_active_task() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;

        timer_service
            .switch_task(task.id.clone(), Some(&task))
            .await
            .unwrap();

        let cmd = StartTimerSessionCmd { task_id: None };

        start_timer_session(&timer_service, &task_repo, &event_publisher, cmd)
            .await
            .unwrap();

        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Running);
        assert_eq!(state.active_task_id, Some(task.id));
    }

    #[tokio::test]
    async fn should_fail_without_active_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;

        let cmd = StartTimerSessionCmd { task_id: None };

        let result = start_timer_session(&timer_service, &task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;

        let cmd = StartTimerSessionCmd {
            task_id: Some("nonexistent-id".to_string()),
        };

        let result = start_timer_session(&timer_service, &task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn should_fail_with_completed_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;

        let mut completed_task = Task::new("Completed Task".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap();
        task_repo.create(completed_task.clone()).await.unwrap();

        let cmd = StartTimerSessionCmd {
            task_id: Some(completed_task.id.to_string()),
        };

        let result = start_timer_session(&timer_service, &task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }

    mod event_tests {
        use super::*;
        use domain::MockEventPublisher;

        async fn setup_with_mock_publisher() -> (
            Arc<dyn TimerService + Send + Sync>,
            Arc<dyn TaskRepository + Send + Sync>,
            Arc<MockEventPublisher>,
            Task,
        ) {
            let timer_service: Arc<dyn TimerService + Send + Sync> =
                Arc::new(MockTimerService::new());
            let task_repo: Arc<dyn TaskRepository + Send + Sync> =
                Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());

            let task = Task::new("Test Task".to_string(), 4).unwrap();
            task_repo.create(task.clone()).await.unwrap();

            (timer_service, task_repo, mock_publisher, task)
        }

        fn create_event_publisher(
            mock: Arc<MockEventPublisher>,
        ) -> Arc<dyn EventPublisher + Send + Sync> {
            mock
        }

        #[tokio::test]
        async fn should_publish_timer_started_event() {
            let (timer_service, task_repo, mock_publisher, task) =
                setup_with_mock_publisher().await;
            let event_publisher = create_event_publisher(mock_publisher.clone());

            let cmd = StartTimerSessionCmd {
                task_id: Some(task.id.to_string()),
            };

            start_timer_session(&timer_service, &task_repo, &event_publisher, cmd)
                .await
                .unwrap();

            // Verify TimerStarted event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("TimerStarted"));
            assert_eq!(mock_publisher.last_event_type().unwrap(), "TimerStarted");
        }

        #[tokio::test]
        async fn should_not_publish_event_on_failure() {
            let (timer_service, task_repo, mock_publisher, _) = setup_with_mock_publisher().await;
            let event_publisher = create_event_publisher(mock_publisher.clone());

            let cmd = StartTimerSessionCmd {
                task_id: Some("nonexistent-id".to_string()),
            };

            let result =
                start_timer_session(&timer_service, &task_repo, &event_publisher, cmd).await;
            assert!(result.is_err());

            // Verify no events were published on failure
            assert_eq!(mock_publisher.event_count(), 0);
        }

        #[tokio::test]
        async fn should_publish_event_with_correct_sequence() {
            let (timer_service, task_repo, mock_publisher, task) =
                setup_with_mock_publisher().await;
            let event_publisher = create_event_publisher(mock_publisher.clone());

            // Start timer, then stop it, then start again to test event sequence
            let cmd = StartTimerSessionCmd {
                task_id: Some(task.id.to_string()),
            };

            // First start
            start_timer_session(&timer_service, &task_repo, &event_publisher, cmd.clone())
                .await
                .unwrap();

            // Stop timer to allow second start
            timer_service.stop_timer().await.unwrap();

            // Second start
            start_timer_session(&timer_service, &task_repo, &event_publisher, cmd)
                .await
                .unwrap();

            // Verify event sequence
            assert_eq!(mock_publisher.event_count(), 2);
            assert!(mock_publisher.verify_event_sequence(&["TimerStarted", "TimerStarted"]));
        }
    }
}
