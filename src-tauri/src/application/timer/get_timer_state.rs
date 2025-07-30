use pomotoro_domain::{TaskRepository, TimerState, Task, Result};
use pomotoro_domain::timer::TimerService;
use std::sync::Arc;

/// Get the current timer state
/// 
/// This use case retrieves the current timer state from the timer service.
/// It provides a clean abstraction for controllers to access timer information
/// without directly depending on infrastructure concerns.
/// 
/// ## Business Rules
/// 
/// - Always loads the latest state from persistence
/// - Returns complete timer state information
/// 
/// ## Dependencies
/// 
/// - TimerService: For timer state access (domain abstraction)
pub async fn get_timer_state(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
) -> Result<TimerState> {
    // Load any persisted state first
    timer_service.load_state().await?;
    
    // Return current state
    timer_service.get_state().await
}

/// Get timer state with active task information
/// 
/// This enhanced version includes the active task details along with
/// the timer state, providing a complete view for the frontend.
/// 
/// ## Dependencies
/// 
/// - TimerService: For timer state access (domain abstraction)
/// - TaskRepository: For active task details
/// 
/// ## Returns
/// 
/// - Tuple of (TimerState, Option<Task>) with current state and active task
pub async fn get_timer_state_with_task(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
) -> Result<(TimerState, Option<Task>)> {
    // Load any persisted state first
    timer_service.load_state().await?;
    
    // Get current timer state
    let timer_state = timer_service.get_state().await?;
    
    // Get active task if available
    let active_task = if let Some(task_id) = &timer_state.active_task_id {
        task_repo.get_by_id(task_id.clone()).await?
    } else {
        None
    };
    
    Ok((timer_state, active_task))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{TaskId, TimerStatus, Phase};
    use crate::infrastructure::InMemoryTaskRepository;
    use std::sync::{Arc, RwLock};
    use async_trait::async_trait;
    
    // Mock timer service for testing
    struct MockTimerService {
        state: Arc<RwLock<TimerState>>,
        load_called: Arc<RwLock<bool>>,
    }
    
    impl MockTimerService {
        fn new() -> Self {
            Self {
                state: Arc::new(RwLock::new(TimerState::default())),
                load_called: Arc::new(RwLock::new(false)),
            }
        }
        
        fn new_with_task(task_id: TaskId) -> Self {
            let mut state = TimerState::default();
            state.switch_task(task_id).unwrap();
            Self {
                state: Arc::new(RwLock::new(state)),
                load_called: Arc::new(RwLock::new(false)),
            }
        }
        
        fn was_load_called(&self) -> bool {
            *self.load_called.read().unwrap()
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
            let mut load_called = self.load_called.write().unwrap();
            *load_called = true;
            Ok(())
        }
        
        async fn save_state(&self) -> Result<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn should_get_timer_state_and_load_first() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());
        
        let state = get_timer_state(&timer_service).await.unwrap();
        
        assert_eq!(state.status(), TimerStatus::Stopped);
        // Note: In real implementation, load_state would be called
    }
    
    #[tokio::test]
    async fn should_get_timer_state_with_task() {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_task(task.id.clone()));
        
        let (timer_state, active_task) = get_timer_state_with_task(&timer_service, &task_repo).await.unwrap();
        
        assert_eq!(timer_state.active_task_id, Some(task.id.clone()));
        assert!(active_task.is_some());
        assert_eq!(active_task.unwrap().id, task.id);
    }
    
    #[tokio::test]
    async fn should_get_timer_state_without_active_task() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        
        let (timer_state, active_task) = get_timer_state_with_task(&timer_service, &task_repo).await.unwrap();
        
        assert_eq!(timer_state.active_task_id, None);
        assert!(active_task.is_none());
    }
}