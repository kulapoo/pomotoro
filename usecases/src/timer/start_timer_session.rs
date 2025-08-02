use domain::{TaskRepository, TaskId, Result, Error};
use domain::timer::TimerService;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StartTimerSessionCmd {
    pub task_id: Option<String>,
}

/// Start a timer session with optional task switching
/// 
/// This use case orchestrates starting a pomodoro timer session,
/// optionally switching to a specific task first. It coordinates
/// between the task repository and timer service to ensure proper
/// business logic execution.
/// 
/// ## Business Rules
/// 
/// - Task must exist and not be completed if task_id is provided
/// - Timer must not already be running
/// - An active task must be available (either provided or existing)
/// 
/// ## Dependencies
/// 
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For task validation and retrieval
pub async fn start_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cmd: StartTimerSessionCmd,
) -> Result<()> {
    let task = if let Some(task_id_str) = cmd.task_id {
        let task_id = TaskId::from_string(&task_id_str)
            .map_err(|_| Error::TaskNotFound { id: task_id_str.clone() })?;
        
        // Verify task exists and is not completed
        let task = task_repo
            .get_by_id(task_id.clone())
            .await?
            .ok_or_else(|| Error::TaskNotFound { id: task_id_str })?;
        
        if task.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }
        
        // Switch to the task first
        timer_service.switch_task(task_id, Some(&task)).await?;
        
        Some(task)
    } else {
        // Get current state to check if we have an active task
        let current_state = timer_service.get_state().await?;
        if current_state.active_task_id.is_none() {
            return Err(Error::InvalidStateTransition {
                from: "no_active_task".to_string(),
                to: "start_session".to_string(),
            });
        }
        
        // Get the active task for context
        if let Some(task_id) = current_state.active_task_id {
            task_repo.get_by_id(task_id).await?
        } else {
            None
        }
    };
    
    // Start the timer with task context
    timer_service.start_timer(task.as_ref()).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Task, TimerState, TimerStatus};
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
        
        async fn skip_to_next_phase(&self, _task: Option<&Task>) -> Result<(domain::Phase, domain::Phase)> {
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
        let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        (timer_service, task_repo, task)
    }
    
    #[tokio::test]
    async fn should_start_timer_session_with_task_id() {
        let (timer_service, task_repo, task) = setup().await;
        
        let cmd = StartTimerSessionCmd {
            task_id: Some(task.id.to_string()),
        };
        
        start_timer_session(&timer_service, &task_repo, cmd).await.unwrap();
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Running);
        assert_eq!(state.active_task_id, Some(task.id));
    }
    
    #[tokio::test]
    async fn should_start_timer_session_with_existing_active_task() {
        let (timer_service, task_repo, task) = setup().await;
        
        // Set up existing active task
        timer_service.switch_task(task.id.clone(), Some(&task)).await.unwrap();
        
        let cmd = StartTimerSessionCmd {
            task_id: None,
        };
        
        start_timer_session(&timer_service, &task_repo, cmd).await.unwrap();
        
        let state = timer_service.get_state().await.unwrap();
        assert_eq!(state.status(), TimerStatus::Running);
        assert_eq!(state.active_task_id, Some(task.id));
    }
    
    #[tokio::test]
    async fn should_fail_without_active_task() {
        let (timer_service, task_repo, _) = setup().await;
        
        let cmd = StartTimerSessionCmd {
            task_id: None,
        };
        
        let result = start_timer_session(&timer_service, &task_repo, cmd).await;
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }
    
    #[tokio::test]
    async fn should_fail_with_nonexistent_task() {
        let (timer_service, task_repo, _) = setup().await;
        
        let cmd = StartTimerSessionCmd {
            task_id: Some("nonexistent-id".to_string()),
        };
        
        let result = start_timer_session(&timer_service, &task_repo, cmd).await;
        assert!(matches!(result, Err(Error::TaskNotFound { .. })));
    }
    
    #[tokio::test]
    async fn should_fail_with_completed_task() {
        let (timer_service, task_repo, _) = setup().await;
        
        let mut completed_task = Task::new("Completed Task".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap();
        task_repo.create(completed_task.clone()).await.unwrap();
        
        let cmd = StartTimerSessionCmd {
            task_id: Some(completed_task.id.to_string()),
        };
        
        let result = start_timer_session(&timer_service, &task_repo, cmd).await;
        assert!(matches!(result, Err(Error::TaskAlreadyCompleted)));
    }
}