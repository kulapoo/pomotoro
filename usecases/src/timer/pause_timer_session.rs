use domain::{TimerStatus, Result, EventPublisher, TimerPaused};
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
    
    // Business logic: Publish TimerPaused event when timer becomes paused
    if new_status == TimerStatus::Paused {
        let state = timer_service.get_state().await?;
        let timer_paused_event = TimerPaused::new(
            state.active_task_id,
            state.timer.phase,
            state.timer.remaining_seconds,
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
mod tests {
    use super::*;
    use domain::{Task, TimerState, TaskId, Phase};
    use std::sync::{Arc, RwLock};
    use async_trait::async_trait;
    
    // Mock timer service for testing
    struct MockTimerService {
        state: Arc<RwLock<TimerState>>,
    }
    
    impl MockTimerService {
        fn new_with_status(status: TimerStatus) -> Self {
            let mut state = TimerState::default();
            // Bypass validation for testing by setting the status directly
            state.timer.status = status;
            Self {
                state: Arc::new(RwLock::new(state)),
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
        
        async fn skip_to_next_phase(&self, _task: Option<&Task>) -> Result<(Phase, Phase)> {
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
            
            // Verify TimerPaused event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("TimerPaused"));
            assert_eq!(mock_publisher.last_event_type().unwrap(), "TimerPaused");
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
            assert!(mock_publisher.verify_event_sequence(&["TimerPaused", "TimerPaused"]));
        }
    }
}