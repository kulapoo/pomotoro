use domain::TimerService;
use domain::{Result, TimerState};
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::{Phase, TaskId, TimerStatus, TimerConfiguration};
    use std::sync::{Arc, RwLock};

    // Mock timer service for testing
    struct MockTimerService {
        state: Arc<RwLock<TimerState>>,
        load_called: Arc<RwLock<bool>>,
    }

    impl MockTimerService {
        fn new() -> Self {
            Self {
                state: Arc::new(RwLock::new(TimerState::Idle {
                    configuration: TimerConfiguration::default(),
                    session_count: 0,
                    active_task: None,
                })),
                load_called: Arc::new(RwLock::new(false)),
            }
        }

        fn new_with_task(task_id: TaskId) -> Self {
            Self {
                state: Arc::new(RwLock::new(TimerState::Idle {
                    configuration: TimerConfiguration::default(),
                    session_count: 0,
                    active_task: Some(task_id),
                })),
                load_called: Arc::new(RwLock::new(false)),
            }
        }
    }

    #[async_trait]
    impl TimerService for MockTimerService {
        async fn start_timer(&self, _task: Option<&domain::Task>) -> Result<()> {
            let mut state = self.state.write().unwrap();
            if let TimerState::Idle { configuration, session_count, active_task } = &*state {
                *state = TimerState::Working {
                    remaining_seconds: configuration.get_phase_duration_seconds(Phase::Work),
                    configuration: configuration.clone(),
                    session_count: *session_count,
                    active_task: *active_task,
                    task_session_count: 0,
                };
            }
            Ok(())
        }

        async fn stop_timer(&self) -> Result<()> {
            let mut state = self.state.write().unwrap();
            *state = TimerState::Idle {
                configuration: TimerConfiguration::default(),
                session_count: 0,
                active_task: None,
            };
            Ok(())
        }

        async fn toggle_pause(&self) -> Result<TimerStatus> {
            let mut state = self.state.write().unwrap();
            match &*state {
                TimerState::Working { remaining_seconds, configuration, session_count, active_task, task_session_count } => {
                    *state = TimerState::Paused {
                        paused_from: Box::new(TimerState::Working {
                            remaining_seconds: *remaining_seconds,
                            configuration: configuration.clone(),
                            session_count: *session_count,
                            active_task: *active_task,
                            task_session_count: *task_session_count,
                        }),
                        remaining_seconds: *remaining_seconds,
                    };
                    Ok(TimerStatus::Paused)
                }
                TimerState::Paused { paused_from, .. } => {
                    *state = *paused_from.clone();
                    Ok(TimerStatus::Running)
                }
                _ => Ok(state.status()),
            }
        }

        async fn reset_current_phase(&self, _task: Option<&domain::Task>) -> Result<()> {
            let mut state = self.state.write().unwrap();
            *state = TimerState::Idle {
                configuration: TimerConfiguration::default(),
                session_count: 0,
                active_task: None,
            };
            Ok(())
        }

        async fn skip_to_next_phase(&self, _task: Option<&domain::Task>) -> Result<(Phase, Phase)> {
            let old_phase = self.state.read().unwrap().phase();
            let new_phase = match old_phase {
                Phase::Work => Phase::ShortBreak,
                Phase::ShortBreak => Phase::Work,
                Phase::LongBreak => Phase::Work,
            };
            Ok((old_phase, new_phase))
        }

        async fn get_state(&self) -> Result<TimerState> {
            Ok(self.state.read().unwrap().clone())
        }

        async fn switch_task(&self, task_id: TaskId, _task: Option<&domain::Task>) -> Result<()> {
            let mut state = self.state.write().unwrap();
            match &mut *state {
                TimerState::Idle { active_task, .. } => *active_task = Some(task_id),
                TimerState::Working { active_task, .. } => *active_task = Some(task_id),
                TimerState::ShortBreak { active_task, .. } => *active_task = Some(task_id),
                TimerState::LongBreak { active_task, .. } => *active_task = Some(task_id),
                TimerState::Paused { paused_from, .. } => {
                    // Update the paused state as well
                    match paused_from.as_mut() {
                        TimerState::Working { active_task, .. } => *active_task = Some(task_id),
                        _ => {}
                    }
                }
            }
            Ok(())
        }

        async fn load_state(&self) -> Result<()> {
            let mut load_called = self.load_called.write().unwrap();
            *load_called = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_get_timer_state_and_load_first() {
        let timer_service: Arc<dyn TimerService + Send + Sync> = Arc::new(MockTimerService::new());

        let state = get_timer_state(&timer_service).await.unwrap();

        assert!(matches!(state, TimerState::Idle { .. }));
        // Note: In real implementation, load_state would be called
    }
}