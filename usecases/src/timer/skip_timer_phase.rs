use domain::{TaskRepository, Phase, Result, EventPublisher, WorkSessionCompleted};
use domain::timer::TimerService;
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
    // Get current state to find active task
    let current_state = timer_service.get_state().await?;
    
    // Get the active task for context
    let task = if let Some(task_id) = current_state.active_task_id {
        task_repo.get_by_id(task_id).await?
    } else {
        None
    };
    
    // Store initial state for business logic decisions
    let old_phase = current_state.timer.phase;
    
    // Skip to the next phase with task context
    let (_, new_phase) = timer_service.skip_to_next_phase(task.as_ref()).await?;
    
    // Business logic: Publish WorkSessionCompleted event if we completed a work session
    if old_phase == Phase::Work && (new_phase == Phase::ShortBreak || new_phase == Phase::LongBreak) {
        if let Some(task_ref) = &task {
            let updated_state = timer_service.get_state().await?;
            let work_session_event = WorkSessionCompleted::new(
                Some(task_ref.id.clone()),
                1500, // 25 minutes work session default duration (TODO: get from task config)
                updated_state.session_count(),
                task_ref.current_sessions as u32 + 1, // increment since we just completed
                1, // version
            );
            event_publisher.publish(Box::new(work_session_event));
        }
    }
    
    Ok((old_phase, new_phase))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Task, TimerState, TimerStatus, TaskId};
    use domain::InMemoryTaskRepository;
    use std::sync::{Arc, RwLock};
    use async_trait::async_trait;
    
    // Mock timer service for testing
    struct MockTimerService {
        state: Arc<RwLock<TimerState>>,
    }
    
    impl MockTimerService {
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
    async fn should_skip_timer_phase_with_active_task() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;
        
        // Get initial phase
        let initial_state = timer_service.get_state().await.unwrap();
        let initial_phase = initial_state.timer.phase;
        
        let (old_phase, new_phase) = skip_timer_phase(&timer_service, &task_repo, &event_publisher).await.unwrap();
        
        assert_eq!(old_phase, initial_phase);
        assert_ne!(old_phase, new_phase);
        
        // Verify the phase actually changed
        let final_state = timer_service.get_state().await.unwrap();
        assert_eq!(final_state.timer.phase, new_phase);
        assert_eq!(final_state.active_task_id, Some(task.id));
    }
    
    #[tokio::test]
    async fn should_skip_timer_phase_without_active_task() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_task(TaskId::new()));
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(domain::NoOpEventPublisher);
        
        // Should not fail even if active task doesn't exist in repo
        let result = skip_timer_phase(&timer_service, &task_repo, &event_publisher).await;
        assert!(result.is_ok());
        
        let (old_phase, new_phase) = result.unwrap();
        assert_ne!(old_phase, new_phase);
    }
    
    // Enhanced event testing for Phase 4
    mod event_tests {
        use super::*;
        use domain::MockEventPublisher;
        
        fn create_event_publisher(mock: Arc<MockEventPublisher>) -> Arc<dyn EventPublisher + Send + Sync> {
            mock
        }
        
        // Enhanced mock service with controllable phase transitions
        struct ControllableMockTimerService {
            state: Arc<RwLock<TimerState>>,
            next_phase: Phase,
        }
        
        impl ControllableMockTimerService {
            fn new_with_task_and_phase(task_id: TaskId, current_phase: Phase, next_phase: Phase) -> Self {
                let mut state = TimerState::default();
                state.switch_task(task_id).unwrap();
                state.timer.phase = current_phase; // Set initial phase
                Self {
                    state: Arc::new(RwLock::new(state)),
                    next_phase,
                }
            }
        }
        
        #[async_trait]
        impl TimerService for ControllableMockTimerService {
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
                let old_phase = state.timer.phase;
                state.timer.phase = self.next_phase;
                Ok((old_phase, self.next_phase))
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
        
        async fn setup_with_mock_publisher_and_phases(
            current_phase: Phase, 
            next_phase: Phase
        ) -> (
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
                Arc::new(ControllableMockTimerService::new_with_task_and_phase(
                    task.id.clone(), current_phase, next_phase
                ));
            
            (timer_service, task_repo, mock_publisher, task)
        }
        
        #[tokio::test]
        async fn should_publish_work_session_completed_when_skipping_from_work_to_short_break() {
            let (timer_service, task_repo, mock_publisher, _) = 
                setup_with_mock_publisher_and_phases(Phase::Work, Phase::ShortBreak).await;
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let (old_phase, new_phase) = skip_timer_phase(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            assert_eq!(old_phase, Phase::Work);
            assert_eq!(new_phase, Phase::ShortBreak);
            
            // Verify WorkSessionCompleted event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("WorkSessionCompleted"));
            assert_eq!(mock_publisher.last_event_type().unwrap(), "WorkSessionCompleted");
        }
        
        #[tokio::test]
        async fn should_publish_work_session_completed_when_skipping_from_work_to_long_break() {
            let (timer_service, task_repo, mock_publisher, _) = 
                setup_with_mock_publisher_and_phases(Phase::Work, Phase::LongBreak).await;
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let (old_phase, new_phase) = skip_timer_phase(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            assert_eq!(old_phase, Phase::Work);
            assert_eq!(new_phase, Phase::LongBreak);
            
            // Verify WorkSessionCompleted event was published
            assert_eq!(mock_publisher.event_count(), 1);
            assert!(mock_publisher.has_event_type("WorkSessionCompleted"));
        }
        
        #[tokio::test]
        async fn should_not_publish_event_when_skipping_from_break_to_work() {
            let (timer_service, task_repo, mock_publisher, _) = 
                setup_with_mock_publisher_and_phases(Phase::ShortBreak, Phase::Work).await;
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let (old_phase, new_phase) = skip_timer_phase(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            assert_eq!(old_phase, Phase::ShortBreak);
            assert_eq!(new_phase, Phase::Work);
            
            // Verify no events were published when transitioning from break to work
            assert_eq!(mock_publisher.event_count(), 0);
        }
        
        #[tokio::test]
        async fn should_not_publish_event_when_skipping_between_breaks() {
            let (timer_service, task_repo, mock_publisher, _) = 
                setup_with_mock_publisher_and_phases(Phase::ShortBreak, Phase::LongBreak).await;
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let (old_phase, new_phase) = skip_timer_phase(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            assert_eq!(old_phase, Phase::ShortBreak);
            assert_eq!(new_phase, Phase::LongBreak);
            
            // Verify no events were published when transitioning between breaks
            assert_eq!(mock_publisher.event_count(), 0);
        }
        
        #[tokio::test]
        async fn should_not_publish_event_when_no_active_task() {
            let timer_service: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(ControllableMockTimerService::new_with_task_and_phase(
                    TaskId::new(), Phase::Work, Phase::ShortBreak
                ));
            let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let (old_phase, new_phase) = skip_timer_phase(&timer_service, &task_repo, &event_publisher).await.unwrap();
            
            assert_eq!(old_phase, Phase::Work);
            assert_eq!(new_phase, Phase::ShortBreak);
            
            // Verify no events were published when task doesn't exist in repo
            assert_eq!(mock_publisher.event_count(), 0);
        }
        
        #[tokio::test]
        async fn should_publish_multiple_work_session_events() {
            // Create two separate timer services to simulate different work sessions
            let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
            let mock_publisher = Arc::new(MockEventPublisher::new());
            let event_publisher = create_event_publisher(mock_publisher.clone());
            
            let task = Task::new("Test Task".to_string(), 4).unwrap();
            task_repo.create(task.clone()).await.unwrap();
            
            // First work session completion
            let timer_service1: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(ControllableMockTimerService::new_with_task_and_phase(
                    task.id.clone(), Phase::Work, Phase::ShortBreak
                ));
            
            skip_timer_phase(&timer_service1, &task_repo, &event_publisher).await.unwrap();
            
            // Second work session completion
            let timer_service2: Arc<dyn TimerService + Send + Sync> = 
                Arc::new(ControllableMockTimerService::new_with_task_and_phase(
                    task.id.clone(), Phase::Work, Phase::ShortBreak
                ));
            
            skip_timer_phase(&timer_service2, &task_repo, &event_publisher).await.unwrap();
            
            // Verify multiple events were published
            assert_eq!(mock_publisher.event_count(), 2);
            assert!(mock_publisher.verify_event_sequence(&["WorkSessionCompleted", "WorkSessionCompleted"]));
        }
    }
}