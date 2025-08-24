use domain::{TimerStatus, Result, EventPublisher, timer::Paused};
use domain::timer::TimerService;
use std::sync::Arc;

/// Pause or resume a timer session
/// 
/// This use case toggles the timer state between running and paused.
/// It uses the timer service abstraction to handle the state transition
/// while maintaining proper business logic.
/// 
/// ## Business Rules
/// 
/// - Can only pause/resume when timer is in Running or Paused state
/// - Returns the new status after the operation
/// 
/// ## Dependencies
/// 
/// - TimerService: For timer state management (domain abstraction)
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn pause_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    let new_status = timer_service.toggle_pause().await?;
    
    // Business logic: Publish Paused event when timer becomes paused
    if new_status == TimerStatus::Paused {
        let state = timer_service.get_state().await?;
        let timer_paused_event = Paused::new(
            state.active_entity_id(),
            state.phase(),
            state.remaining_seconds(),
            1, // version
        );
        event_publisher.publish(Box::new(timer_paused_event));
    }
    
    Ok(new_status)
}

/// Resume a paused timer session
/// 
/// This is an alias for pause_timer_session for better semantic clarity
/// when the intent is specifically to resume a paused timer.
pub async fn resume_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    timer_service.toggle_pause().await
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use domain::{TimerState, TaskId, Phase, TimerConfiguration, TimerStatus};
    use std::sync::{Arc, RwLock};
    use async_trait::async_trait;
    
    // Mock timer service for testing
    struct MockTimerService {
        state: Arc<RwLock<TimerState>>,
    }
    
    impl MockTimerService {
        fn new_with_status(status: TimerStatus) -> Self {
            let state = match status {
                TimerStatus::Running => TimerState::Working {
                    remaining_seconds: 1500,
                    configuration: TimerConfiguration::default(),
                    session_count: 0,
                    active_entity: Some(TaskId::new().to_string()),
                    entity_session_count: 0,
                },
                TimerStatus::Paused => TimerState::Paused {
                    paused_from: Box::new(TimerState::Working {
                        remaining_seconds: 1500,
                        configuration: TimerConfiguration::default(),
                        session_count: 0,
                        active_entity: Some(TaskId::new().to_string()),
                        entity_session_count: 0,
                    }),
                    remaining_seconds: 1500,
                },
                _ => TimerState::default(),
            };
            Self {
                state: Arc::new(RwLock::new(state)),
            }
        }
    }
    
    #[async_trait]
    impl TimerService for MockTimerService {
        async fn start_timer(&self, _task: Option<&domain::Task>) -> Result<()> {
            let state = self.state.read().unwrap();
            if let TimerState::Idle { configuration, session_count, active_entity } = &*state {
                let new_state = TimerState::Working {
                    remaining_seconds: configuration.get_phase_duration_seconds(Phase::Work),
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
                (state.configuration().clone(), state.active_entity().map(|s| s.to_string()))
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
                TimerState::Working { .. } | TimerState::ShortBreak { .. } | TimerState::LongBreak { .. } => {
                    TimerState::Paused {
                        paused_from: Box::new(state.clone()),
                        remaining_seconds: state.remaining_seconds(),
                    }
                }
                TimerState::Paused { paused_from, .. } => {
                    *paused_from.clone()
                }
                _ => state.clone(),
            };
            let status = new_state.status();
            drop(state);
            *self.state.write().unwrap() = new_state;
            Ok(status)
        }
        
        async fn reset_current_phase(&self, task: Option<&domain::Task>) -> Result<()> {
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
        
        async fn skip_to_next_phase(&self, task: Option<&domain::Task>) -> Result<(Phase, Phase)> {
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
        
        async fn switch_task(&self, task_id: TaskId, _task: Option<&domain::Task>) -> Result<()> {
            let state = self.state.read().unwrap();
            if let TimerState::Idle { configuration, session_count, .. } = &*state {
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
    
    #[tokio::test]
    async fn should_pause_running_timer() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_status(TimerStatus::Running));
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(domain::NoOpEventPublisher);
        
        let result = pause_timer_session(&timer_service, &event_publisher).await.unwrap();
        assert_eq!(result, TimerStatus::Paused);
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Paused);
    }
    
    #[tokio::test]
    async fn should_resume_paused_timer() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_status(TimerStatus::Paused));
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(domain::NoOpEventPublisher);
        
        let result = resume_timer_session(&timer_service, &event_publisher).await.unwrap();
        assert_eq!(result, TimerStatus::Running);
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Running);
    }
    
    #[tokio::test]
    async fn should_handle_stopped_timer() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_status(TimerStatus::Stopped));
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(domain::NoOpEventPublisher);
        
        let result = pause_timer_session(&timer_service, &event_publisher).await.unwrap();
        assert_eq!(result, TimerStatus::Stopped);
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Stopped);
    }
    
    // Enhanced event testing for Phase 4
    mod event_tests {
        use super::*;
        use domain::MockEventPublisher;
        
        fn create_event_publisher(mock: Arc<MockEventPublisher>) -> Arc<dyn EventPublisher + Send + Sync> {
            mock
        }
        
        #[tokio::test]
        async fn should_publish_timer_paused_event_when_pausing() {
            let timer_service: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(MockTimerService::new_with_status(TimerStatus::Running));
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let result = pause_timer_session(&timer_service, &event_publisher).await.unwrap();
            assert_eq!(result, TimerStatus::Paused);
            
            // Verify Paused event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("Paused"));
            assert_eq!(mock_publisher.last_event_type().unwrap(), "Paused");
        }
        
        #[tokio::test]
        async fn should_not_publish_event_when_resuming() {
            let timer_service: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(MockTimerService::new_with_status(TimerStatus::Paused));
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let result = pause_timer_session(&timer_service, &event_publisher).await.unwrap();
            assert_eq!(result, TimerStatus::Running);
            
            // Verify no events were published when resuming (only when pausing)
            assert_eq!(mock_publisher.event_count(), 0);
        }
        
        #[tokio::test]
        async fn should_not_publish_event_when_timer_stopped() {
            let timer_service: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(MockTimerService::new_with_status(TimerStatus::Stopped));
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let result = pause_timer_session(&timer_service, &event_publisher).await.unwrap();
            assert_eq!(result, TimerStatus::Stopped);
            
            // Verify no events were published for stopped timer
            assert_eq!(mock_publisher.event_count(), 0);
        }
        
        #[tokio::test]
        async fn should_publish_events_for_multiple_pause_operations() {
            let timer_service: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(MockTimerService::new_with_status(TimerStatus::Running));
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            // First pause (Running -> Paused)
            pause_timer_session(&timer_service, &event_publisher).await.unwrap();
            assert_eq!(mock_publisher.event_count(), 1);
            
            // Resume (Paused -> Running) - no event
            pause_timer_session(&timer_service, &event_publisher).await.unwrap();
            assert_eq!(mock_publisher.event_count(), 1); // Still 1
            
            // Pause again (Running -> Paused) - another event
            pause_timer_session(&timer_service, &event_publisher).await.unwrap();
            assert_eq!(mock_publisher.event_count(), 2);
            
            // Verify sequence
            assert!(mock_publisher.verify_event_sequence(&["Paused", "Paused"]));
        }
    }
}