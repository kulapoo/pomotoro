use domain::{TaskRepository, Result, EventPublisher, TimerReset};
use domain::timer::TimerService;
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
    // Get current state to find active task
    let current_state = timer_service.get_state().await?;
    
    // Get the active task for context
    let task = if let Some(task_id) = current_state.active_task_id {
        task_repo.get_by_id(task_id).await?
    } else {
        None
    };
    
    // Reset the current phase with task context
    timer_service.reset_current_phase(task.as_ref()).await?;
    
    // Business logic: Publish TimerReset event after successful reset
    let updated_state = timer_service.get_state().await?;
    let timer_reset_event = TimerReset::new(
        updated_state.active_task_id,
        updated_state.timer.phase,
        1, // version
    );
    event_publisher.publish(Box::new(timer_reset_event));
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Task, TimerState, TimerStatus, TaskId, Phase};
    use domain::InMemoryTaskRepository;
    use std::sync::{Arc, RwLock};
    use async_trait::async_trait;
    
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
            let mut state = TimerState::default();
            state.switch_task(task_id).unwrap();
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
    
    async fn setup() -> (
        Arc<dyn TimerService + Send + Sync>, 
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
        Task
    ) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(domain::NoOpEventPublisher);
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_task(task.id.clone()));
        
        (timer_service, task_repo, event_publisher, task)
    }
    
    #[tokio::test]
    async fn should_reset_timer_session_with_active_task() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;
        
        // Modify timer state to simulate partial progress
        {
            let state = timer_service.get_state().await.unwrap();
            assert_eq!(state.active_task_id, Some(task.id));
        }
        
        reset_timer_session(&timer_service, &task_repo, &event_publisher).await.unwrap();
        
        // Verify reset was called (in real implementation, this would reset remaining time)
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.active_task_id, Some(task.id));
    }
    
    #[tokio::test]
    async fn should_reset_timer_session_without_active_task() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(domain::NoOpEventPublisher);
        
        // Should not fail even without active task
        reset_timer_session(&timer_service, &task_repo, &event_publisher).await.unwrap();
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.active_task_id, None);
    }
    
    // Enhanced event testing for Phase 4
    mod event_tests {
        use super::*;
        use domain::MockEventPublisher;
        
        fn create_event_publisher(mock: Arc<MockEventPublisher>) -> Arc<dyn EventPublisher + Send + Sync> {
            mock
        }
        
        async fn setup_with_mock_publisher() -> (
            Arc<dyn TimerService + Send + Sync>, 
            Arc<dyn TaskRepository + Send + Sync>,
            Arc<MockEventPublisher>,
            Task
        ) {
            let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());
            
            let task = Task::new("Test Task".to_string(), 4).unwrap();
            task_repo.create(task.clone()).await.unwrap();
            
            let timer_service: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(MockTimerService::new_with_task(task.id.clone()));
            
            (timer_service, task_repo, mock_publisher, task)
        }
        
        #[tokio::test]
        async fn should_publish_timer_reset_event() {
            let (timer_service, task_repo, mock_publisher, _) = setup_with_mock_publisher().await;
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            reset_timer_session(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            // Verify TimerReset event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("TimerReset"));
            assert_eq!(mock_publisher.last_event_type().unwrap(), "TimerReset");
        }
        
        #[tokio::test]
        async fn should_publish_event_even_without_active_task() {
            let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());
            let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            reset_timer_session(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            // Verify TimerReset event was published even without active task
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("TimerReset"));
        }
        
        #[tokio::test]
        async fn should_publish_events_for_multiple_resets() {
            let (timer_service, task_repo, mock_publisher, _) = setup_with_mock_publisher().await;
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            // Reset timer multiple times
            reset_timer_session(&timer_service, &task_repo, &event_publisher).await.unwrap();
            reset_timer_session(&timer_service, &task_repo, &event_publisher).await.unwrap();
            reset_timer_session(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            // Verify multiple events were published
            assert_eq!(mock_publisher.event_count(), 3);
            assert!(mock_publisher.verify_event_sequence(&["TimerReset", "TimerReset", "TimerReset"]));
        }
    }
}