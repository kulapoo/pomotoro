use domain::{TaskRepository, TaskId, Result, Error, EventPublisher};
use domain::timer::TimerService;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SwitchTimerTaskCmd {
    pub task_id: String,
}

/// Switch the active task for the timer
/// 
/// This use case changes the active task context for the timer,
/// potentially adjusting timer configuration based on task-specific
/// settings. It validates the task exists and is not completed.
/// 
/// ## Business Rules
/// 
/// - Task must exist and not be completed
/// - Switches task context while preserving timer state where appropriate
/// - May adjust timer configuration based on task settings
/// 
/// ## Dependencies
/// 
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For task validation and retrieval
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn switch_timer_task(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: SwitchTimerTaskCmd,
) -> Result<()> {
    let task_id = TaskId::from_string(&cmd.task_id)
        .map_err(|_| Error::TaskNotFound { id: cmd.task_id.clone() })?;
    
    // Verify task exists and is not completed
    let task = task_repo
        .get_by_id(task_id.clone())
        .await?
        .ok_or_else(|| Error::TaskNotFound { id: cmd.task_id })?;
    
    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }
    
    // Switch to the task with its configuration
    timer_service.switch_task(task_id, Some(&task)).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Task, TimerState, TimerStatus, Phase};
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
        let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(domain::NoOpEventPublisher);
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        (timer_service, task_repo, event_publisher, task)
    }
    
    #[tokio::test]
    async fn should_switch_timer_task_successfully() {
        let (timer_service, task_repo, event_publisher, task) = setup().await;
        
        let cmd = SwitchTimerTaskCmd {
            task_id: task.id.to_string(),
        };
        
        switch_timer_task(&timer_service, &task_repo, &event_publisher, cmd).await.unwrap();
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.active_task_id, Some(task.id));
    }
    
    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;
        
        let cmd = SwitchTimerTaskCmd {
            task_id: "nonexistent-id".to_string(),
        };
        
        let result = switch_timer_task(&timer_service, &task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }
    
    #[tokio::test]
    async fn should_fail_with_completed_task() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;
        
        let mut completed_task = Task::new("Completed Task".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap();
        task_repo.create(completed_task.clone()).await.unwrap();
        
        let cmd = SwitchTimerTaskCmd {
            task_id: completed_task.id.to_string(),
        };
        
        let result = switch_timer_task(&timer_service, &task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }
    
    #[tokio::test]
    async fn should_fail_with_invalid_task_id() {
        let (timer_service, task_repo, event_publisher, _) = setup().await;
        
        let cmd = SwitchTimerTaskCmd {
            task_id: "".to_string(), // Empty string should be invalid
        };
        
        let result = switch_timer_task(&timer_service, &task_repo, &event_publisher, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }
}