use super::{Phase, Error, Result};
use super::state_machine::TimerState;

/// Result of a state transition
#[derive(Debug, Clone)]
pub struct TransitionResult {
    /// The new state after transition
    pub new_state: TimerState,
    /// The phase that was completed (if applicable)
    pub completed_phase: Option<Phase>,
    /// Whether a work session was completed
    pub work_session_completed: bool,
    /// Whether a full pomodoro cycle was completed
    pub cycle_completed: bool,
}

/// State transition logic for the timer state machine
pub struct StateTransitions;

impl StateTransitions {
    /// Start the timer from idle state
    pub fn start(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Idle { configuration, session_count, active_entity } => {
                if active_entity.is_none() {
                    return Err(Error::NoActiveEntity);
                }
                
                let remaining_seconds = configuration.get_phase_duration_seconds(Phase::Work);
                
                Ok(TransitionResult {
                    new_state: TimerState::Working {
                        remaining_seconds,
                        configuration,
                        session_count,
                        active_entity,
                        entity_session_count: 0,
                    },
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{:?}", state),
                to: "Start".to_string(),
            }),
        }
    }
    
    /// Pause the timer
    pub fn pause(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Working { remaining_seconds, .. } |
            TimerState::ShortBreak { remaining_seconds, .. } |
            TimerState::LongBreak { remaining_seconds, .. } => {
                Ok(TransitionResult {
                    new_state: TimerState::Paused {
                        paused_from: Box::new(state.clone()),
                        remaining_seconds,
                    },
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                })
            }
            TimerState::Paused { .. } => {
                // Already paused, no-op
                Ok(TransitionResult {
                    new_state: state,
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                })
            }
            TimerState::Idle { .. } => Err(Error::InvalidStateTransition {
                from: "Stopped".to_string(),
                to: "Pause".to_string(),
            }),
        }
    }
    
    /// Resume from paused state
    pub fn resume(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Paused { paused_from, .. } => {
                Ok(TransitionResult {
                    new_state: *paused_from,
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{:?}", state),
                to: "Resume".to_string(),
            }),
        }
    }
    
    /// Reset the timer to idle
    pub fn reset(state: TimerState) -> Result<TransitionResult> {
        let configuration = state.configuration().clone();
        let active_entity = state.active_entity().map(|s| s.to_string());
        let session_count = match &state {
            // Preserve session count when resetting from break
            TimerState::ShortBreak { session_count, .. } |
            TimerState::LongBreak { session_count, .. } => *session_count,
            // Reset session count for other states
            _ => 0,
        };
        
        Ok(TransitionResult {
            new_state: TimerState::Idle {
                configuration,
                session_count,
                active_entity,
            },
            completed_phase: None,
            work_session_completed: false,
            cycle_completed: false,
        })
    }
    
    /// Transition to the next phase when current phase completes
    pub fn complete_phase(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Working { configuration, session_count, active_entity, entity_session_count, .. } => {
                let new_session_count = session_count + 1;
                let new_entity_session_count = entity_session_count + 1;
                
                // Determine next phase based on session count
                let sessions_until_long = configuration.sessions_until_long_break as u32;
                let (next_state, cycle_completed) = if new_session_count % sessions_until_long == 0 {
                    // Time for long break
                    let remaining_seconds = configuration.get_phase_duration_seconds(Phase::LongBreak);
                    (
                        TimerState::LongBreak {
                            remaining_seconds,
                            configuration,
                            session_count: new_session_count,
                            active_entity,
                            entity_session_count: new_entity_session_count,
                        },
                        true,
                    )
                } else {
                    // Time for short break
                    let remaining_seconds = configuration.get_phase_duration_seconds(Phase::ShortBreak);
                    (
                        TimerState::ShortBreak {
                            remaining_seconds,
                            configuration,
                            session_count: new_session_count,
                            active_entity,
                            entity_session_count: new_entity_session_count,
                        },
                        false,
                    )
                };
                
                Ok(TransitionResult {
                    new_state: next_state,
                    completed_phase: Some(Phase::Work),
                    work_session_completed: true,
                    cycle_completed,
                })
            }
            TimerState::ShortBreak { configuration, session_count, active_entity, entity_session_count, .. } => {
                // Return to work
                let remaining_seconds = configuration.get_phase_duration_seconds(Phase::Work);
                
                Ok(TransitionResult {
                    new_state: TimerState::Working {
                        remaining_seconds,
                        configuration,
                        session_count,
                        active_entity,
                        entity_session_count, // Preserve entity sessions when returning from break
                    },
                    completed_phase: Some(Phase::ShortBreak),
                    work_session_completed: false,
                    cycle_completed: false,
                })
            }
            TimerState::LongBreak { configuration, session_count, active_entity, entity_session_count, .. } => {
                // Return to work, potentially reset session count
                let remaining_seconds = configuration.get_phase_duration_seconds(Phase::Work);
                let reset_sessions = session_count >= configuration.sessions_until_long_break as u32;
                
                Ok(TransitionResult {
                    new_state: TimerState::Working {
                        remaining_seconds,
                        configuration,
                        session_count: if reset_sessions { 0 } else { session_count },
                        active_entity,
                        entity_session_count, // Preserve entity sessions when returning from break
                    },
                    completed_phase: Some(Phase::LongBreak),
                    work_session_completed: false,
                    cycle_completed: false,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{:?}", state),
                to: "CompletePhase".to_string(),
            }),
        }
    }
    
    /// Skip the current phase and move to the next
    pub fn skip_phase(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Working { .. } |
            TimerState::ShortBreak { .. } |
            TimerState::LongBreak { .. } => {
                // Skipping is like completing with 0 time remaining
                Self::complete_phase(state)
            }
            TimerState::Paused { paused_from, .. } => {
                // Unpause and then skip
                Self::skip_phase(*paused_from)
            }
            TimerState::Idle { .. } => Err(Error::InvalidStateTransition {
                from: "Stopped".to_string(),
                to: "Skip".to_string(),
            }),
        }
    }
    
    /// Process a timer tick (decrement time)
    pub fn tick(mut state: TimerState) -> Result<(TimerState, bool)> {
        let phase_complete = match &mut state {
            TimerState::Working { remaining_seconds, .. } |
            TimerState::ShortBreak { remaining_seconds, .. } |
            TimerState::LongBreak { remaining_seconds, .. } => {
                if *remaining_seconds > 0 {
                    *remaining_seconds -= 1;
                    *remaining_seconds == 0
                } else {
                    false
                }
            }
            _ => false,
        };
        
        Ok((state, phase_complete))
    }
    
    /// Switch the active entity (only allowed in idle state)
    pub fn switch_entity(state: TimerState, new_entity: Option<String>) -> Result<TransitionResult> {
        match state {
            TimerState::Idle { configuration, session_count, .. } => {
                Ok(TransitionResult {
                    new_state: TimerState::Idle {
                        configuration,
                        session_count,
                        active_entity: new_entity,
                    },
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{:?}", state),
                to: "SwitchEntity".to_string(),
            }),
        }
    }
    
    /// Check if a transition is valid from the current state
    pub fn can_transition(from: &TimerState, transition: TransitionType) -> bool {
        match (from, transition) {
            (TimerState::Idle { active_entity, .. }, TransitionType::Start) => active_entity.is_some(),
            (TimerState::Idle { .. }, TransitionType::Reset) => true,
            (TimerState::Idle { .. }, TransitionType::SwitchTask) => true,
            
            (TimerState::Working { .. }, TransitionType::Pause) => true,
            (TimerState::Working { .. }, TransitionType::Reset) => true,
            (TimerState::Working { .. }, TransitionType::Skip) => true,
            (TimerState::Working { remaining_seconds, .. }, TransitionType::Complete) => *remaining_seconds == 0,
            
            (TimerState::ShortBreak { .. }, TransitionType::Pause) => true,
            (TimerState::ShortBreak { .. }, TransitionType::Reset) => true,
            (TimerState::ShortBreak { .. }, TransitionType::Skip) => true,
            (TimerState::ShortBreak { remaining_seconds, .. }, TransitionType::Complete) => *remaining_seconds == 0,
            
            (TimerState::LongBreak { .. }, TransitionType::Pause) => true,
            (TimerState::LongBreak { .. }, TransitionType::Reset) => true,
            (TimerState::LongBreak { .. }, TransitionType::Skip) => true,
            (TimerState::LongBreak { remaining_seconds, .. }, TransitionType::Complete) => *remaining_seconds == 0,
            
            (TimerState::Paused { .. }, TransitionType::Resume) => true,
            (TimerState::Paused { .. }, TransitionType::Reset) => true,
            (TimerState::Paused { .. }, TransitionType::Skip) => true,
            
            _ => false,
        }
    }
}

/// Types of transitions that can be performed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionType {
    Start,
    Pause,
    Resume,
    Reset,
    Complete,
    Skip,
    SwitchTask,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimerConfiguration;
    
    fn create_entity_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    #[test]
    fn should_start_timer_from_idle() {
        let entity_id = create_entity_id();
        let state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(entity_id),
        };
        
        let result = StateTransitions::start(state).unwrap();
        assert!(matches!(result.new_state, TimerState::Working { .. }));
        assert!(!result.work_session_completed);
    }
    
    #[test]
    fn should_not_start_without_entity() {
        let state = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        };
        
        let result = StateTransitions::start(state);
        assert!(result.is_err());
    }
    
    #[test]
    fn should_pause_running_timer() {
        let state = TimerState::Working {
            remaining_seconds: 100,
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(create_entity_id()),
            entity_session_count: 0,
        };
        
        let result = StateTransitions::pause(state).unwrap();
        assert!(matches!(result.new_state, TimerState::Paused { .. }));
    }
    
    #[test]
    fn should_resume_paused_timer() {
        let working = TimerState::Working {
            remaining_seconds: 100,
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(create_entity_id()),
            entity_session_count: 0,
        };
        
        let paused = TimerState::Paused {
            paused_from: Box::new(working.clone()),
            remaining_seconds: 100,
        };
        
        let result = StateTransitions::resume(paused).unwrap();
        assert!(matches!(result.new_state, TimerState::Working { .. }));
    }
    
    #[test]
    fn should_transition_from_work_to_short_break() {
        let state = TimerState::Working {
            remaining_seconds: 0,
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(create_entity_id()),
            entity_session_count: 0,
        };
        
        let result = StateTransitions::complete_phase(state).unwrap();
        assert!(matches!(result.new_state, TimerState::ShortBreak { .. }));
        assert_eq!(result.completed_phase, Some(Phase::Work));
        assert!(result.work_session_completed);
        assert!(!result.cycle_completed);
    }
    
    #[test]
    fn should_transition_to_long_break_after_cycle() {
        let mut config = TimerConfiguration::default();
        config.sessions_until_long_break = 2;
        
        let state = TimerState::Working {
            remaining_seconds: 0,
            configuration: config,
            session_count: 1, // Will be 2 after completion
            active_entity: Some(create_entity_id()),
            entity_session_count: 0,
        };
        
        let result = StateTransitions::complete_phase(state).unwrap();
        assert!(matches!(result.new_state, TimerState::LongBreak { .. }));
        assert!(result.cycle_completed);
    }
    
    #[test]
    fn should_process_tick() {
        let state = TimerState::Working {
            remaining_seconds: 2,
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(create_entity_id()),
            entity_session_count: 0,
        };
        
        let (new_state, complete) = StateTransitions::tick(state).unwrap();
        assert!(!complete);
        assert_eq!(new_state.remaining_seconds(), 1);
        
        let (final_state, complete) = StateTransitions::tick(new_state).unwrap();
        assert!(complete);
        assert_eq!(final_state.remaining_seconds(), 0);
    }
    
    #[test]
    fn should_validate_transitions() {
        let idle = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(create_entity_id()),
        };
        
        assert!(StateTransitions::can_transition(&idle, TransitionType::Start));
        assert!(!StateTransitions::can_transition(&idle, TransitionType::Pause));
        
        let working = TimerState::Working {
            remaining_seconds: 100,
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(create_entity_id()),
            entity_session_count: 0,
        };
        
        assert!(StateTransitions::can_transition(&working, TransitionType::Pause));
        assert!(!StateTransitions::can_transition(&working, TransitionType::Start));
    }
}