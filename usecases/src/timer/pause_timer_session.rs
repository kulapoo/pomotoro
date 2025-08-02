use domain::{TimerStatus, Result};
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
pub async fn pause_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
) -> Result<TimerStatus> {
    timer_service.toggle_pause().await
}

/// Resume a paused timer session
/// 
/// This is an alias for pause_timer_session for better semantic clarity
/// when the intent is specifically to resume a paused timer.
pub async fn resume_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
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
        
        let result = pause_timer_session(&timer_service).await.unwrap();
        assert_eq!(result, TimerStatus::Paused);
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Paused);
    }
    
    #[tokio::test]
    async fn should_resume_paused_timer() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_status(TimerStatus::Paused));
        
        let result = resume_timer_session(&timer_service).await.unwrap();
        assert_eq!(result, TimerStatus::Running);
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Running);
    }
    
    #[tokio::test]
    async fn should_handle_stopped_timer() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_status(TimerStatus::Stopped));
        
        let result = pause_timer_session(&timer_service).await.unwrap();
        assert_eq!(result, TimerStatus::Stopped);
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Stopped);
    }
}