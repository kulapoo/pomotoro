use pomotoro_domain::{TaskRepository, Phase, Result};
use pomotoro_domain::timer::TimerService;
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
/// 
/// ## Returns
/// 
/// - Tuple of (old_phase, new_phase) indicating the transition
pub async fn skip_timer_phase(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
) -> Result<(Phase, Phase)> {
    // Get current state to find active task
    let current_state = timer_service.get_state().await?;
    
    // Get the active task for context
    let task = if let Some(task_id) = current_state.active_task_id {
        task_repo.get_by_id(task_id).await?
    } else {
        None
    };
    
    // Skip to the next phase with task context
    let (old_phase, new_phase) = timer_service.skip_to_next_phase(task.as_ref()).await?;
    
    Ok((old_phase, new_phase))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{Task, TimerState, TimerStatus, TaskId};
    use crate::infrastructure::InMemoryTaskRepository;
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
    
    async fn setup() -> (Arc<dyn TimerService + Send + Sync>, Arc<dyn TaskRepository + Send + Sync>, Task) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        let timer_service: Arc<dyn TimerService + Send + Sync> = 
            Arc::new(MockTimerService::new_with_task(task.id.clone()));
        
        (timer_service, task_repo, task)
    }
    
    #[tokio::test]
    async fn should_skip_timer_phase_with_active_task() {
        let (timer_service, task_repo, task) = setup().await;
        
        // Get initial phase
        let initial_state = timer_service.get_state().await.unwrap();
        let initial_phase = initial_state.timer.phase;
        
        let (old_phase, new_phase) = skip_timer_phase(&timer_service, &task_repo).await.unwrap();
        
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
        
        // Should not fail even if active task doesn't exist in repo
        let result = skip_timer_phase(&timer_service, &task_repo).await;
        assert!(result.is_ok());
        
        let (old_phase, new_phase) = result.unwrap();
        assert_ne!(old_phase, new_phase);
    }
}