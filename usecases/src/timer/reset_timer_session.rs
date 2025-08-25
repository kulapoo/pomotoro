use domain::timer::TimerService;
use domain::{EventPublisher, Result, TaskId, TaskRepository, timer::Reset};
use std::sync::Arc;

/// Reset the current timer session phase
///
/// This use case resets the current timer phase back to its full duration
/// while preserving the current phase and task context. It coordinates
/// between the timer service and task repository to provide proper context.
///
/// ## Business Rules
///
/// - Resets only the current phase, not the entire session
/// - Preserves the active task and phase context
/// - Can be called in any timer state
///
/// ## Dependencies
///
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For active task context
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn reset_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let current_state = timer_service.get_state().await?;

    let task = if let Some(entity_id_str) = current_state.active_entity_id() {
        if let Ok(task_id) = TaskId::from_string(&entity_id_str) {
            task_repo.get_by_id(task_id).await?
        } else {
            None
        }
    } else {
        None
    };

    // Reset the current phase with task context
    timer_service.reset_current_phase(task.as_ref()).await?;

    // Business logic: Publish Reset event after successful reset
    let updated_state = timer_service.get_state().await?;
    let timer_reset_event = Reset::new(
        updated_state.active_entity_id(),
        updated_state.phase(),
        1, // version
    );
    event_publisher.publish(Box::new(timer_reset_event));

    Ok(())
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::InMemoryTaskRepository;
    use domain::{
        Phase, Task, TaskId, TimerConfiguration, TimerState, TimerStatus,
    };
    use std::sync::{Arc, RwLock};

    // Mock timer service for testing
    struct MockTimerService {
        state: Arc<RwLock<TimerState>>,
    }

    impl MockTimerService {
        fn new() -> Self {
            Self {
                state: Arc::new(RwLock::new(TimerState::default())),
            }
        }

        fn new_with_task(task_id: TaskId) -> Self {
            let state = TimerState::Idle {
                configuration: TimerConfiguration::default(),
                session_count: 0,
                active_entity: Some(task_id.to_string()),
            };
            Self {
                state: Arc::new(RwLock::new(state)),
            }
        }
    }

    #[async_trait]
    impl TimerService for MockTimerService {
        async fn start_timer(
            &self,
            _task: Option<&domain::Task>,
        ) -> Result<()> {
            let state = self.state.read().unwrap();
            if let TimerState::Idle {
                configuration,
                session_count,
                active_entity,
            } = &*state
            {
                let new_state = TimerState::Working {
                    remaining_seconds: configuration
                        .get_phase_duration_seconds(Phase::Work),
                    configuration: configuration.clone(),
                    session_count: *session_count,
                    active_entity: active_entity.clone(),
                    entity_session_count: 0,
                };
                drop(state);
                *self.state.write().unwrap() = new_state;
            }
            Ok(())
        }

        async fn stop_timer(&self) -> Result<()> {
            let (config, active_entity) = {
                let state = self.state.read().unwrap();
                (
                    state.configuration().clone(),
                    state.active_entity().map(|s| s.to_string()),
                )
            };
            *self.state.write().unwrap() = TimerState::Idle {
                configuration: config,
                session_count: 0,
                active_entity,
            };
            Ok(())
        }

        async fn toggle_pause(&self) -> Result<TimerStatus> {
            let state = self.state.read().unwrap();
            let new_state = match &*state {
                TimerState::Working { .. }
                | TimerState::ShortBreak { .. }
                | TimerState::LongBreak { .. } => TimerState::Paused {
                    paused_from: Box::new(state.clone()),
                    remaining_seconds: state.remaining_seconds(),
                },
                TimerState::Paused { paused_from, .. } => *paused_from.clone(),
                _ => state.clone(),
            };
            let status = new_state.status();
            drop(state);
            *self.state.write().unwrap() = new_state;
            Ok(status)
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
        ) -> Result<(Phase, Phase)> {
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
            let state = self.state.read().unwrap();
            if let TimerState::Idle {
                configuration,
                session_count,
                ..
            } = &*state
            {
                let new_state = TimerState::Idle {
                    configuration: configuration.clone(),
                    session_count: *session_count,
                    active_entity: Some(task_id.to_string()),
                };
                drop(state);
                *self.state.write().unwrap() = new_state;
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
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(domain::NoOpEventPublisher);

        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();

        let timer_service: Arc<dyn TimerService + Send + Sync> =
            Arc::new(MockTimerService::new_with_task(task.id));

        (timer_service, task_repo, event_publisher, task)
    }

    #[tokio::test]
    async fn should_reset_timer_session_with_active_task() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;

        // Modify timer state to simulate partial progress
        {
            let state = timer_service.get_state().await.unwrap();
            assert_eq!(state.active_entity_id(), Some(task.id.to_string()));
        }

        reset_timer_session(&timer_service, &task_repo, &event_publisher)
            .await
            .unwrap();

        // Verify reset was called (in real implementation, this would reset remaining time)
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.active_entity_id(), Some(task.id.to_string()));
    }

    #[tokio::test]
    async fn should_reset_timer_session_without_active_task() {
        let timer_service: Arc<dyn TimerService + Send + Sync> =
            Arc::new(MockTimerService::new());
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(domain::NoOpEventPublisher);

        // Should not fail even without active task
        reset_timer_session(&timer_service, &task_repo, &event_publisher)
            .await
            .unwrap();

        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.active_entity_id(), None);
    }

    // Enhanced event testing for Phase 4
    mod event_tests {
        use super::*;
        use domain::MockEventPublisher;

        fn create_event_publisher(
            mock: Arc<MockEventPublisher>,
        ) -> Arc<dyn EventPublisher + Send + Sync> {
            mock
        }

        async fn setup_with_mock_publisher() -> (
            Arc<dyn TimerService + Send + Sync>,
            Arc<dyn TaskRepository + Send + Sync>,
            Arc<MockEventPublisher>,
            Task,
        ) {
            let task_repo: Arc<dyn TaskRepository + Send + Sync> =
                Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());

            let task = Task::new("Test Task".to_string(), 4).unwrap();
            task_repo.create(task.clone()).await.unwrap();

            let timer_service: Arc<dyn TimerService + Send + Sync> =
                Arc::new(MockTimerService::new_with_task(task.id));

            (timer_service, task_repo, mock_publisher, task)
        }

        #[tokio::test]
        async fn should_publish_timer_reset_event() {
            let (timer_service, task_repo, mock_publisher, _) =
                setup_with_mock_publisher().await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            reset_timer_session(&timer_service, &task_repo, &event_publisher)
                .await
                .unwrap();

            // Verify Reset event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("Reset"));
            assert_eq!(mock_publisher.last_event_type().unwrap(), "Reset");
        }

        #[tokio::test]
        async fn should_publish_event_even_without_active_task() {
            let timer_service: Arc<dyn TimerService + Send + Sync> =
                Arc::new(MockTimerService::new());
            let task_repo: Arc<dyn TaskRepository + Send + Sync> =
                Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            reset_timer_session(&timer_service, &task_repo, &event_publisher)
                .await
                .unwrap();

            // Verify Reset event was published even without active task
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("Reset"));
        }

        #[tokio::test]
        async fn should_publish_events_for_multiple_resets() {
            let (timer_service, task_repo, mock_publisher, _) =
                setup_with_mock_publisher().await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            // Reset timer multiple times
            reset_timer_session(&timer_service, &task_repo, &event_publisher)
                .await
                .unwrap();
            reset_timer_session(&timer_service, &task_repo, &event_publisher)
                .await
                .unwrap();
            reset_timer_session(&timer_service, &task_repo, &event_publisher)
                .await
                .unwrap();

            // Verify multiple events were published
            assert_eq!(mock_publisher.event_count(), 3);
            assert!(
                mock_publisher
                    .verify_event_sequence(&["Reset", "Reset", "Reset"])
            );
        }
    }
}
