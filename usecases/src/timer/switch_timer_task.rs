use domain::{TaskRepository, TaskId, Result, Error, EventPublisher};
use super::TimerService;
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
        .get_by_id(task_id)
        .await?
        .ok_or(Error::TaskNotFound { id: cmd.task_id })?;
    
    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }
    
    // Switch to the task with its configuration
    timer_service.switch_task(task_id, Some(&task)).await?;
    
    Ok(())
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use domain::{Task, TimerState, TimerStatus, Phase, TimerConfiguration};
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
            let state = self.state.read().unwrap();
            let config = state.configuration().clone();
            let active_entity = state.active_entity_id();
            drop(state);
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
        assert_eq!(state.active_entity_id(), Some(task.id.to_string()));
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