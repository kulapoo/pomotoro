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

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::InMemoryTaskRepository;
    use domain::{Phase, Task, TimerConfiguration, TimerState, TimerStatus};
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
        async fn start_timer(
            &self,
            _task: Option<&domain::Task>,
        ) -> Result<()> {
            let mut state = self.state.write().unwrap();
            // Mock implementation: transition to Working state
            if let TimerState::Idle {
                configuration,
                session_count,
                active_entity,
            } = state.clone()
            {
                if active_entity.is_some() {
                    *state = TimerState::Working {
                        remaining_seconds: configuration
                            .get_phase_duration_seconds(domain::Phase::Work),
                        configuration,
                        session_count,
                        active_entity,
                        entity_session_count: 0,
                    };
                }
            }
            Ok(())
        }

        async fn stop_timer(&self) -> Result<()> {
            let mut state = self.state.write().unwrap();
            // Mock implementation: transition to Idle state
            let configuration = state.configuration().clone();
            let active_entity = state.active_entity_id();
            *state = TimerState::Idle {
                configuration,
                session_count: 0,
                active_entity,
            };
            Ok(())
        }

        async fn toggle_pause(&self) -> Result<TimerStatus> {
            let mut state = self.state.write().unwrap();
            match state.clone() {
                TimerState::Working {
                    remaining_seconds, ..
                }
                | TimerState::ShortBreak {
                    remaining_seconds, ..
                }
                | TimerState::LongBreak {
                    remaining_seconds, ..
                } => {
                    // Pause the timer
                    *state = TimerState::Paused {
                        paused_from: Box::new(state.clone()),
                        remaining_seconds,
                    };
                    Ok(TimerStatus::Paused)
                }
                TimerState::Paused { paused_from, .. } => {
                    // Resume the timer
                    *state = *paused_from;
                    Ok(TimerStatus::Running)
                }
                _ => Ok(state.status()),
            }
        }

        async fn reset_current_phase(
            &self,
            task: Option<&domain::Task>,
        ) -> Result<()> {
            let mut state = self.state.write().unwrap();
            // Reset to a new work phase
            *state = TimerState::Working {
                remaining_seconds: 1500,
                configuration: TimerConfiguration::default(),
                session_count: 0,
                active_entity: task.map(|t| t.id().to_string()),
                entity_session_count: 0,
            };
            Ok(())
        }

        async fn skip_to_next_phase(
            &self,
            task: Option<&domain::Task>,
        ) -> Result<(domain::Phase, domain::Phase)> {
            let mut state = self.state.write().unwrap();
            let old_phase = state.phase();
            let task_id = task.map(|t| t.id().to_string());

            // Transition to next phase based on current phase
            let new_phase = match old_phase {
                Phase::Work => Phase::ShortBreak,
                Phase::ShortBreak => Phase::Work,
                Phase::LongBreak => Phase::Work,
            };

            *state = match new_phase {
                Phase::Work => TimerState::Working {
                    remaining_seconds: 1500,
                    configuration: TimerConfiguration::default(),
                    session_count: state.session_count() + 1,
                    active_entity: task_id.map(|id| id.to_string()),
                    entity_session_count: 0,
                },
                Phase::ShortBreak => TimerState::ShortBreak {
                    remaining_seconds: 300,
                    configuration: TimerConfiguration::default(),
                    session_count: state.session_count(),
                    active_entity: task_id.map(|id| id.to_string()),
                    entity_session_count: 0,
                },
                Phase::LongBreak => TimerState::LongBreak {
                    remaining_seconds: 900,
                    configuration: TimerConfiguration::default(),
                    session_count: state.session_count(),
                    active_entity: task_id.map(|id| id.to_string()),
                    entity_session_count: 0,
                },
            };

            Ok((old_phase, new_phase))
        }

        async fn get_state(&self) -> Result<TimerState> {
            Ok(self.state.read().unwrap().clone())
        }

        async fn switch_task(
            &self,
            task_id: TaskId,
            _task: Option<&domain::Task>,
        ) -> Result<()> {
            let mut state = self.state.write().unwrap();
            // Mock implementation: only allow task switch when idle
            if let TimerState::Idle {
                configuration,
                session_count,
                ..
            } = state.clone()
            {
                *state = TimerState::Idle {
                    configuration,
                    session_count,
                    active_entity: Some(task_id.to_string()),
                };
            }
            Ok(())
        }

        async fn load_state(&self) -> Result<()> {
            Ok(())
        }
    }

    async fn setup() -> (
        Arc<dyn TimerService + Send + Sync>,
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
        Task,
    ) {
        let timer_service: Arc<dyn TimerService + Send + Sync> =
            Arc::new(MockTimerService::new());
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
        assert_eq!(state.active_entity_id(), Some(task.id.to_string()));
    }

    #[tokio::test]
    async fn should_start_timer_session_with_existing_active_task() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;

        timer_service
            .switch_task(task.id, Some(&task))
            .await
            .unwrap();

        let cmd = StartTimerSessionCmd { task_id: None };

        start_timer_session(&timer_service, &task_repo, &event_publisher, cmd)
            .await
            .unwrap();

        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Running);
        assert_eq!(state.active_entity_id(), Some(task.id.to_string()));
    }

    #[tokio::test]
    async fn should_fail_without_active_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;

        let cmd = StartTimerSessionCmd { task_id: None };

        let result = start_timer_session(
            &timer_service,
            &task_repo,
            &event_publisher,
            cmd,
        )
        .await;
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;

        let cmd = StartTimerSessionCmd {
            task_id: Some("nonexistent-id".to_string()),
        };

        let result = start_timer_session(
            &timer_service,
            &task_repo,
            &event_publisher,
            cmd,
        )
        .await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn should_fail_with_completed_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;

        let mut completed_task =
            Task::new("Completed Task".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap();
        task_repo.create(completed_task.clone()).await.unwrap();

        let cmd = StartTimerSessionCmd {
            task_id: Some(completed_task.id.to_string()),
        };

        let result = start_timer_session(
            &timer_service,
            &task_repo,
            &event_publisher,
            cmd,
        )
        .await;
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
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let cmd = StartTimerSessionCmd {
                task_id: Some(task.id.to_string()),
            };

            start_timer_session(
                &timer_service,
                &task_repo,
                &event_publisher,
                cmd,
            )
            .await
            .unwrap();

            // Verify Started event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("Started"));
            assert_eq!(mock_publisher.last_event_type().unwrap(), "Started");
        }

        #[tokio::test]
        async fn should_not_publish_event_on_failure() {
            let (timer_service, task_repo, mock_publisher, _) =
                setup_with_mock_publisher().await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let cmd = StartTimerSessionCmd {
                task_id: Some("nonexistent-id".to_string()),
            };

            let result = start_timer_session(
                &timer_service,
                &task_repo,
                &event_publisher,
                cmd,
            )
            .await;
            assert!(result.is_err());

            // Verify no events were published on failure
            assert_eq!(mock_publisher.event_count(), 0);
        }

        #[tokio::test]
        async fn should_publish_event_with_correct_sequence() {
            let (timer_service, task_repo, mock_publisher, task) =
                setup_with_mock_publisher().await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            // Start timer, then stop it, then start again to test event sequence
            let cmd = StartTimerSessionCmd {
                task_id: Some(task.id.to_string()),
            };

            // First start
            start_timer_session(
                &timer_service,
                &task_repo,
                &event_publisher,
                cmd.clone(),
            )
            .await
            .unwrap();

            // Stop timer to allow second start
            timer_service.stop_timer().await.unwrap();

            // Second start
            start_timer_session(
                &timer_service,
                &task_repo,
                &event_publisher,
                cmd,
            )
            .await
            .unwrap();

            // Verify event sequence
            assert_eq!(mock_publisher.event_count(), 2);
            assert!(
                mock_publisher.verify_event_sequence(&["Started", "Started"])
            );
        }
    }
}
