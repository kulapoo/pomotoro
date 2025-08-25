use domain::timer::TimerService;
use domain::{
    EventPublisher, Phase, Result, TaskId, TaskRepository, WorkSessionCompleted,
};
use std::sync::Arc;

/// Skip to the next phase in the pomodoro cycle
///
/// This use case immediately transitions the timer to the next phase
/// (work -> break -> work) and handles any necessary work session completion
/// events. It coordinates between the timer service and task repository.
///
/// ## Business Rules
///
/// - Transitions immediately to next phase regardless of remaining time
/// - May trigger work session completion events if skipping work phase
/// - Follows standard pomodoro cycle progression
///
/// ## Dependencies
///
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For active task context
/// - EventPublisher: For domain event publishing (business orchestration)
///
/// ## Returns
///
/// - Tuple of (old_phase, new_phase) indicating the transition
pub async fn skip_timer_phase(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<(Phase, Phase)> {
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

    // Store initial state for business logic decisions
    let old_phase = current_state.phase();

    // Skip to the next phase with task context
    let (_, new_phase) =
        timer_service.skip_to_next_phase(task.as_ref()).await?;

    // Business logic: Publish WorkSessionCompleted event if we completed a work session
    if old_phase == Phase::Work
        && (new_phase == Phase::ShortBreak || new_phase == Phase::LongBreak)
    {
        if let Some(task_ref) = &task {
            let updated_state = timer_service.get_state().await?;
            let work_session_event = WorkSessionCompleted::new(
                Some(task_ref.id.to_string()),
                1500, // 25 minutes work session default duration (TODO: get from task config)
                updated_state.session_count(),
                task_ref.current_sessions as u32 + 1, // increment since we just completed
                1,                                    // version
            );
            event_publisher.publish(Box::new(work_session_event));
        }
    }

    Ok((old_phase, new_phase))
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
            let new_state = match state.clone() {
                TimerState::Idle {
                    configuration,
                    session_count,
                    active_entity,
                } => TimerState::Working {
                    remaining_seconds: configuration.work_duration.as_secs()
                        as u32,
                    configuration,
                    session_count,
                    active_entity,
                    entity_session_count: 0,
                },
                _ => state.clone(),
            };
            *self.state.write().unwrap() = new_state;
            Ok(())
        }

        async fn stop_timer(&self) -> Result<()> {
            let state = self.state.read().unwrap();
            let config = state.configuration();
            let active_entity = state.active_entity_id();
            *self.state.write().unwrap() = TimerState::Idle {
                configuration: config.clone(),
                session_count: 0,
                active_entity,
            };
            Ok(())
        }

        async fn toggle_pause(&self) -> Result<TimerStatus> {
            let state = self.state.read().unwrap();
            let new_state = match state.clone() {
                TimerState::Working {
                    remaining_seconds,
                    configuration,
                    session_count,
                    active_entity,
                    entity_session_count,
                } => TimerState::Paused {
                    paused_from: Box::new(TimerState::Working {
                        remaining_seconds,
                        configuration,
                        session_count,
                        active_entity,
                        entity_session_count,
                    }),
                    remaining_seconds,
                },
                TimerState::Paused { paused_from, .. } => *paused_from.clone(),
                _ => state.clone(),
            };
            let status = new_state.status();
            *self.state.write().unwrap() = new_state;
            Ok(status)
        }

        async fn reset_current_phase(
            &self,
            _task: Option<&domain::Task>,
        ) -> Result<()> {
            let state = self.state.read().unwrap();
            let config = state.configuration();
            let active_entity = state.active_entity_id();
            *self.state.write().unwrap() = TimerState::Idle {
                configuration: config.clone(),
                session_count: 0,
                active_entity,
            };
            Ok(())
        }

        async fn skip_to_next_phase(
            &self,
            _task: Option<&domain::Task>,
        ) -> Result<(Phase, Phase)> {
            let mut state = self.state.write().unwrap();
            let old_phase = state.phase();
            let config = state.configuration().clone();
            let session_count = state.session_count();
            let active_entity = state.active_entity_id();

            let new_phase = match old_phase {
                Phase::Work => Phase::ShortBreak,
                Phase::ShortBreak => Phase::Work,
                Phase::LongBreak => Phase::Work,
            };

            // Actually transition the state
            *state = match new_phase {
                Phase::Work => TimerState::Working {
                    remaining_seconds: config
                        .get_phase_duration_seconds(Phase::Work),
                    configuration: config,
                    session_count: if old_phase == Phase::Work {
                        session_count + 1
                    } else {
                        session_count
                    },
                    active_entity,
                    entity_session_count: 0,
                },
                Phase::ShortBreak => TimerState::ShortBreak {
                    remaining_seconds: config
                        .get_phase_duration_seconds(Phase::ShortBreak),
                    configuration: config,
                    session_count: if old_phase == Phase::Work {
                        session_count + 1
                    } else {
                        session_count
                    },
                    active_entity,
                    entity_session_count: 0,
                },
                Phase::LongBreak => TimerState::LongBreak {
                    remaining_seconds: config
                        .get_phase_duration_seconds(Phase::LongBreak),
                    configuration: config,
                    session_count: if old_phase == Phase::Work {
                        session_count + 1
                    } else {
                        session_count
                    },
                    active_entity,
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
            let new_state = match state.clone() {
                TimerState::Idle {
                    configuration,
                    session_count,
                    ..
                } => TimerState::Idle {
                    configuration,
                    session_count,
                    active_entity: Some(task_id.to_string()),
                },
                _ => state.clone(),
            };
            *self.state.write().unwrap() = new_state;
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
    async fn should_skip_timer_phase_with_active_entity() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;

        // Get initial phase
        let initial_state = timer_service.get_state().await.unwrap();
        let initial_phase = initial_state.phase();

        let (old_phase, new_phase) =
            skip_timer_phase(&timer_service, &task_repo, &event_publisher)
                .await
                .unwrap();

        assert_eq!(old_phase, initial_phase);
        assert_ne!(old_phase, new_phase);

        // Verify the phase actually changed
        let final_state = timer_service.get_state().await.unwrap();
        assert_eq!(final_state.phase(), new_phase);
        assert_eq!(final_state.active_entity_id(), Some(task.id.to_string()));
    }

    #[tokio::test]
    async fn should_skip_timer_phase_without_active_entity() {
        let timer_service: Arc<dyn TimerService + Send + Sync> =
            Arc::new(MockTimerService::new_with_task(TaskId::new()));
        let task_repo: Arc<dyn TaskRepository + Send + Sync> =
            Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(domain::NoOpEventPublisher);

        // Should not fail even if active task doesn't exist in repo
        let result =
            skip_timer_phase(&timer_service, &task_repo, &event_publisher)
                .await;
        assert!(result.is_ok());

        let (old_phase, new_phase) = result.unwrap();
        assert_ne!(old_phase, new_phase);
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

        // Enhanced mock service with controllable phase transitions
        struct ControllableMockTimerService {
            state: Arc<RwLock<TimerState>>,
            next_phase: Phase,
        }

        impl ControllableMockTimerService {
            fn new_with_task_and_phase(
                task_id: TaskId,
                current_phase: Phase,
                next_phase: Phase,
            ) -> Self {
                let state = match current_phase {
                    Phase::Work => TimerState::Working {
                        remaining_seconds: 1500,
                        configuration: TimerConfiguration::default(),
                        session_count: 0,
                        active_entity: Some(task_id.to_string()),
                        entity_session_count: 0,
                    },
                    Phase::ShortBreak => TimerState::ShortBreak {
                        remaining_seconds: 300,
                        configuration: TimerConfiguration::default(),
                        session_count: 0,
                        active_entity: Some(task_id.to_string()),
                        entity_session_count: 0,
                    },
                    Phase::LongBreak => TimerState::LongBreak {
                        remaining_seconds: 900,
                        configuration: TimerConfiguration::default(),
                        session_count: 0,
                        active_entity: Some(task_id.to_string()),
                        entity_session_count: 0,
                    },
                };
                Self {
                    state: Arc::new(RwLock::new(state)),
                    next_phase,
                }
            }
        }

        #[async_trait]
        impl TimerService for ControllableMockTimerService {
            async fn start_timer(&self, task: Option<&Task>) -> Result<()> {
                let mut state = self.state.write().unwrap();
                *state = TimerState::Working {
                    remaining_seconds: 1500,
                    configuration: TimerConfiguration::default(),
                    session_count: 0,
                    active_entity: task.map(|t| t.id().to_string()),
                    entity_session_count: 0,
                };
                Ok(())
            }

            async fn stop_timer(&self) -> Result<()> {
                let mut state = self.state.write().unwrap();
                *state = TimerState::Idle {
                    configuration: TimerConfiguration::default(),
                    session_count: 0,
                    active_entity: None,
                };
                Ok(())
            }

            async fn toggle_pause(&self) -> Result<TimerStatus> {
                let state = self.state.read().unwrap();
                match &*state {
                    TimerState::Working { .. } => Ok(TimerStatus::Running),
                    TimerState::ShortBreak { .. }
                    | TimerState::LongBreak { .. } => Ok(TimerStatus::Running),
                    TimerState::Idle { .. } => Ok(TimerStatus::Idle),
                    _ => Ok(TimerStatus::Idle),
                }
            }

            async fn reset_current_phase(
                &self,
                task: Option<&Task>,
            ) -> Result<()> {
                let mut state = self.state.write().unwrap();
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
                task: Option<&Task>,
            ) -> Result<(Phase, Phase)> {
                let mut state = self.state.write().unwrap();
                let old_phase = state.phase();
                let task_id = task.map(|t| t.id().to_string());
                *state = match self.next_phase {
                    Phase::Work => TimerState::Working {
                        remaining_seconds: 1500,
                        configuration: TimerConfiguration::default(),
                        session_count: 0,
                        active_entity: task_id.map(|id| id.to_string()),
                        entity_session_count: 0,
                    },
                    Phase::ShortBreak => TimerState::ShortBreak {
                        remaining_seconds: 300,
                        configuration: TimerConfiguration::default(),
                        session_count: 0,
                        active_entity: task_id.map(|id| id.to_string()),
                        entity_session_count: 0,
                    },
                    Phase::LongBreak => TimerState::LongBreak {
                        remaining_seconds: 900,
                        configuration: TimerConfiguration::default(),
                        session_count: 0,
                        active_entity: task_id.map(|id| id.to_string()),
                        entity_session_count: 0,
                    },
                };
                Ok((old_phase, self.next_phase))
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

        async fn setup_with_mock_publisher_and_phases(
            current_phase: Phase,
            next_phase: Phase,
        ) -> (
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

            let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(
                ControllableMockTimerService::new_with_task_and_phase(
                    task.id,
                    current_phase,
                    next_phase,
                ),
            );

            (timer_service, task_repo, mock_publisher, task)
        }

        #[tokio::test]
        async fn should_publish_work_session_completed_when_skipping_from_work_to_short_break()
         {
            let (timer_service, task_repo, mock_publisher, _) =
                setup_with_mock_publisher_and_phases(
                    Phase::Work,
                    Phase::ShortBreak,
                )
                .await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let (old_phase, new_phase) =
                skip_timer_phase(&timer_service, &task_repo, &event_publisher)
                    .await
                    .unwrap();

            assert_eq!(old_phase, Phase::Work);
            assert_eq!(new_phase, Phase::ShortBreak);

            // Verify WorkSessionCompleted event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("WorkSessionCompleted"));
            assert_eq!(
                mock_publisher.last_event_type().unwrap(),
                "WorkSessionCompleted"
            );
        }

        #[tokio::test]
        async fn should_publish_work_session_completed_when_skipping_from_work_to_long_break()
         {
            let (timer_service, task_repo, mock_publisher, _) =
                setup_with_mock_publisher_and_phases(
                    Phase::Work,
                    Phase::LongBreak,
                )
                .await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let (old_phase, new_phase) =
                skip_timer_phase(&timer_service, &task_repo, &event_publisher)
                    .await
                    .unwrap();

            assert_eq!(old_phase, Phase::Work);
            assert_eq!(new_phase, Phase::LongBreak);

            // Verify WorkSessionCompleted event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("WorkSessionCompleted"));
        }

        #[tokio::test]
        async fn should_not_publish_event_when_skipping_from_break_to_work() {
            let (timer_service, task_repo, mock_publisher, _) =
                setup_with_mock_publisher_and_phases(
                    Phase::ShortBreak,
                    Phase::Work,
                )
                .await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let (old_phase, new_phase) =
                skip_timer_phase(&timer_service, &task_repo, &event_publisher)
                    .await
                    .unwrap();

            assert_eq!(old_phase, Phase::ShortBreak);
            assert_eq!(new_phase, Phase::Work);

            // Verify no events were published when transitioning from break to work
            assert_eq!(mock_publisher.event_count(), 0);
        }

        #[tokio::test]
        async fn should_not_publish_event_when_skipping_between_breaks() {
            let (timer_service, task_repo, mock_publisher, _) =
                setup_with_mock_publisher_and_phases(
                    Phase::ShortBreak,
                    Phase::LongBreak,
                )
                .await;
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let (old_phase, new_phase) =
                skip_timer_phase(&timer_service, &task_repo, &event_publisher)
                    .await
                    .unwrap();

            assert_eq!(old_phase, Phase::ShortBreak);
            assert_eq!(new_phase, Phase::LongBreak);

            // Verify no events were published when transitioning between breaks
            assert_eq!(mock_publisher.event_count(), 0);
        }

        #[tokio::test]
        async fn should_not_publish_event_when_no_active_entity() {
            let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(
                ControllableMockTimerService::new_with_task_and_phase(
                    TaskId::new(),
                    Phase::Work,
                    Phase::ShortBreak,
                ),
            );
            let task_repo: Arc<dyn TaskRepository + Send + Sync> =
                Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let (old_phase, new_phase) =
                skip_timer_phase(&timer_service, &task_repo, &event_publisher)
                    .await
                    .unwrap();

            assert_eq!(old_phase, Phase::Work);
            assert_eq!(new_phase, Phase::ShortBreak);

            // Verify no events were published when task doesn't exist in repo
            assert_eq!(mock_publisher.event_count(), 0);
        }

        #[tokio::test]
        async fn should_publish_multiple_work_session_events() {
            // Create two separate timer services to simulate different work sessions
            let task_repo: Arc<dyn TaskRepository + Send + Sync> =
                Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher =
                create_event_publisher(mock_publisher.clone());

            let task = Task::new("Test Task".to_string(), 4).unwrap();
            task_repo.create(task.clone()).await.unwrap();

            // First work session completion
            let timer_service1: Arc<dyn TimerService + Send + Sync> = Arc::new(
                ControllableMockTimerService::new_with_task_and_phase(
                    task.id,
                    Phase::Work,
                    Phase::ShortBreak,
                ),
            );

            skip_timer_phase(&timer_service1, &task_repo, &event_publisher)
                .await
                .unwrap();

            // Second work session completion
            let timer_service2: Arc<dyn TimerService + Send + Sync> = Arc::new(
                ControllableMockTimerService::new_with_task_and_phase(
                    task.id,
                    Phase::Work,
                    Phase::ShortBreak,
                ),
            );

            skip_timer_phase(&timer_service2, &task_repo, &event_publisher)
                .await
                .unwrap();

            // Verify multiple events were published
            assert_eq!(mock_publisher.event_count(), 2);
            assert!(mock_publisher.verify_event_sequence(&[
                "WorkSessionCompleted",
                "WorkSessionCompleted"
            ]));
        }
    }
}
