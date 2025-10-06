//! Timer state transition logic.
//!
//! This module encapsulates all state transition rules and validation,
//! ensuring valid state changes and generating appropriate domain events.

use super::events::{
    BreakSessionCompleted, BreakSessionStarted, Paused, PhaseCompleted,
    PhaseSkipped, Reset, Started, WorkSessionCompleted, WorkSessionStarted,
};
use super::state_machine::TimerState;
use super::{Error, Phase, Result, TimerId};
use crate::{Event, TaskId, TimerConfiguration};

/// Result of a state transition containing new state and generated events.
#[derive(Debug)]
pub struct TransitionResult {
    pub new_state: TimerState,
    pub events: Vec<Box<dyn Event>>,
}

/// Stateless service containing all timer state transition logic.
pub struct StateTransitions;

impl StateTransitions {
    /// Transitions from Idle to Working state.
    ///
    /// # Errors
    /// Returns error if not in Idle state.
    pub fn start(
        state: TimerState,
        timer_id: TimerId,
        configuration: &TimerConfiguration,
        active_task_id: Option<TaskId>,
    ) -> Result<TransitionResult> {
        match state {
            TimerState::Idle => {
                let remaining_seconds =
                    configuration.work_duration.as_secs() as u32;
                let duration = configuration.work_duration.as_secs() as u32;

                let events: Vec<Box<dyn Event>> = vec![
                    Box::new(
                        Started::new(timer_id, Phase::Work, duration, 1)
                            .with_active_entity(active_task_id),
                    ),
                    Box::new(WorkSessionStarted::new(
                        timer_id, duration, 1,
                    )),
                ];

                Ok(TransitionResult {
                    new_state: TimerState::Working { remaining_seconds },
                    events,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{state:?}"),
                to: "Start".to_string(),
            }),
        }
    }

    /// Transitions from any running state to Paused.
    ///
    /// # Errors
    /// Returns error if in Idle state.
    pub fn pause(
        state: TimerState,
        timer_id: TimerId,
        configuration: &TimerConfiguration,
    ) -> Result<TransitionResult> {
        match state {
            TimerState::Working {
                remaining_seconds, ..
            }
            | TimerState::ShortBreak {
                remaining_seconds, ..
            }
            | TimerState::LongBreak {
                remaining_seconds, ..
            } => {
                let phase = Self::get_phase_from_state(&state);

                let events: Vec<Box<dyn Event>> = vec![
                    Box::new(Paused::new(
                        timer_id,
                        phase,
                        remaining_seconds,
                        1,
                        configuration.clone(),
                    )),
                ];

                Ok(TransitionResult {
                    new_state: TimerState::Paused {
                        paused_from: Box::new(state.clone()),
                        remaining_seconds,
                    },
                    events,
                })
            }
            TimerState::Paused { .. } => Ok(TransitionResult {
                new_state: state,
                events: vec![],
            }),
            TimerState::Idle => Err(Error::InvalidStateTransition {
                from: "Stopped".to_string(),
                to: "Pause".to_string(),
            }),
        }
    }

    /// Resumes from Paused state to previous running state.
    ///
    /// # Errors
    /// Returns error if not paused.
    pub fn resume(
        state: TimerState,
        timer_id: TimerId,
        _configuration: &TimerConfiguration,
        active_task_id: Option<TaskId>,
    ) -> Result<TransitionResult> {
        match state {
            TimerState::Paused { paused_from, remaining_seconds } => {
                let phase = Self::get_phase_from_state(&paused_from);

                // Use the remaining_seconds from the Paused state, not from paused_from
                let events: Vec<Box<dyn Event>> =
                    vec![Box::new(
                        Started::new(timer_id, phase, remaining_seconds, 1)
                            .with_active_entity(active_task_id)
                    )];

                // Create the resumed state with the correct remaining seconds
                let resumed_state = match *paused_from {
                    TimerState::Working { .. } => TimerState::Working { remaining_seconds },
                    TimerState::ShortBreak { .. } => TimerState::ShortBreak { remaining_seconds },
                    TimerState::LongBreak { .. } => TimerState::LongBreak { remaining_seconds },
                    _ => return Err(Error::InvalidStateTransition {
                        from: "Paused".to_string(),
                        to: "Resume".to_string(),
                    }),
                };

                Ok(TransitionResult {
                    new_state: resumed_state,
                    events,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{state:?}"),
                to: "Resume".to_string(),
            }),
        }
    }

    /// Resets timer to Idle state from any state.
    pub fn reset(
        _state: TimerState,
        timer_id: TimerId,
        _configuration: &TimerConfiguration,
    ) -> Result<TransitionResult> {
        let events: Vec<Box<dyn Event>> =
            vec![Box::new(Reset::new(timer_id, Phase::Work, 1))];

        Ok(TransitionResult {
            new_state: TimerState::Idle,
            events,
        })
    }

    /// Completes current phase and transitions to next.
    ///
    /// Generates appropriate session completion and start events.
    pub fn complete_phase(
        state: TimerState,
        timer_id: TimerId,
        configuration: &TimerConfiguration,
        next_phase: Phase,
    ) -> Result<TransitionResult> {
        let from_phase = Self::get_phase_from_state(&state);

        let duration = configuration.get_phase_duration_seconds(next_phase);
        let next_state = match next_phase {
            Phase::Work => TimerState::Working {
                remaining_seconds: duration,
            },
            Phase::ShortBreak => TimerState::ShortBreak {
                remaining_seconds: duration,
            },
            Phase::LongBreak => TimerState::LongBreak {
                remaining_seconds: duration,
            },
        };

        let mut events: Vec<Box<dyn Event>> = vec![Box::new(
            PhaseCompleted::new(timer_id, from_phase, next_phase, 0, 1),
        )];

        match from_phase {
            Phase::Work => {
                events.push(Box::new(WorkSessionCompleted::new(
                    timer_id,
                    configuration.get_phase_duration_seconds(Phase::Work),
                    1,
                )));
                events.push(Box::new(BreakSessionStarted::new(
                    timer_id, next_phase, duration, 1,
                )));
            }
            Phase::ShortBreak | Phase::LongBreak => {
                events.push(Box::new(BreakSessionCompleted::new(
                    timer_id,
                    from_phase,
                    configuration.get_phase_duration_seconds(from_phase),
                    1,
                )));
                if next_phase == Phase::Work {
                    events.push(Box::new(WorkSessionStarted::new(
                        timer_id, duration, 1,
                    )));
                }
            }
        }

        Ok(TransitionResult {
            new_state: next_state,
            events,
        })
    }

    /// Skips current phase and starts next phase.
    ///
    /// Similar to complete_phase but generates PhaseSkipped event.
    pub fn skip_phase(
        state: TimerState,
        timer_id: TimerId,
        configuration: &TimerConfiguration,
        active_task_id: Option<TaskId>,
    ) -> Result<TransitionResult> {
        match state {
            TimerState::Working { .. }
            | TimerState::ShortBreak { .. }
            | TimerState::LongBreak { .. } => {
                let from_phase = Self::get_phase_from_state(&state);

                let next_phase = match from_phase {
                    Phase::Work => Phase::ShortBreak,
                    Phase::ShortBreak | Phase::LongBreak => Phase::Work,
                };

                let mut result = Self::complete_phase(
                    state.clone(),
                    timer_id,
                    configuration,
                    next_phase,
                )?;

                result.events.retain(|event| {
                    let event_type = event.event_type();
                    event_type != "PhaseCompleted"
                        && event_type != "WorkSessionCompleted"
                });

                // Get the duration for the new phase
                let duration = configuration.get_phase_duration_seconds(next_phase);

                // Insert PhaseSkipped and Started events at the beginning
                result.events.insert(
                    0,
                    Box::new(PhaseSkipped::new(
                        timer_id, from_phase, next_phase, 1,
                    )),
                );
                result.events.insert(
                    1,
                    Box::new(
                        Started::new(timer_id, next_phase, duration, 1)
                            .with_active_entity(active_task_id)
                    ),
                );

                Ok(result)
            }
            TimerState::Paused { paused_from, .. } => {
                Self::skip_phase(*paused_from, timer_id, configuration, active_task_id)
            }
            TimerState::Idle => Err(Error::InvalidStateTransition {
                from: "Stopped".to_string(),
                to: "Skip".to_string(),
            }),
        }
    }

    /// Decrements timer by one second.
    ///
    /// Returns updated state and whether phase completed.
    pub fn tick(
        mut state: TimerState,
        _timer_id: TimerId,
        _configuration: &TimerConfiguration,
    ) -> Result<(TimerState, bool)> {
        let phase_complete = match &mut state {
            TimerState::Working {
                remaining_seconds, ..
            }
            | TimerState::ShortBreak {
                remaining_seconds, ..
            }
            | TimerState::LongBreak {
                remaining_seconds, ..
            } => {
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

    /// Extracts the phase from any timer state.
    fn get_phase_from_state(state: &TimerState) -> Phase {
        match state {
            TimerState::Idle => Phase::Work,
            TimerState::Working { .. } => Phase::Work,
            TimerState::ShortBreak { .. } => Phase::ShortBreak,
            TimerState::LongBreak { .. } => Phase::LongBreak,
            TimerState::Paused { paused_from, .. } => {
                Self::get_phase_from_state(paused_from)
            }
        }
    }

    /// Validates if a transition is allowed from current state.
    pub fn can_transition(
        from: &TimerState,
        transition: TransitionType,
    ) -> bool {
        match (from, transition) {
            (TimerState::Idle, TransitionType::Start) => true,
            (TimerState::Idle, TransitionType::Reset) => true,

            (TimerState::Working { .. }, TransitionType::Pause) => true,
            (TimerState::Working { .. }, TransitionType::Reset) => true,
            (TimerState::Working { .. }, TransitionType::Skip) => true,
            (TimerState::Working { .. }, TransitionType::CompletePhase) => true,
            (
                TimerState::Working {
                    remaining_seconds, ..
                },
                TransitionType::Complete,
            ) => *remaining_seconds == 0,

            (TimerState::ShortBreak { .. }, TransitionType::Pause) => true,
            (TimerState::ShortBreak { .. }, TransitionType::Reset) => true,
            (TimerState::ShortBreak { .. }, TransitionType::Skip) => true,
            (TimerState::ShortBreak { .. }, TransitionType::CompletePhase) => {
                true
            }
            (
                TimerState::ShortBreak {
                    remaining_seconds, ..
                },
                TransitionType::Complete,
            ) => *remaining_seconds == 0,

            (TimerState::LongBreak { .. }, TransitionType::Pause) => true,
            (TimerState::LongBreak { .. }, TransitionType::Reset) => true,
            (TimerState::LongBreak { .. }, TransitionType::Skip) => true,
            (TimerState::LongBreak { .. }, TransitionType::CompletePhase) => {
                true
            }
            (
                TimerState::LongBreak {
                    remaining_seconds, ..
                },
                TransitionType::Complete,
            ) => *remaining_seconds == 0,

            (TimerState::Paused { .. }, TransitionType::Resume) => true,
            (TimerState::Paused { .. }, TransitionType::Reset) => true,
            (TimerState::Paused { .. }, TransitionType::Skip) => true,

            _ => false,
        }
    }
}

/// Types of state transitions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionType {
    /// Start timer from idle.
    Start,
    /// Pause running timer.
    Pause,
    /// Resume paused timer.
    Resume,
    /// Reset to idle.
    Reset,
    /// Complete current phase (remaining_seconds == 0).
    Complete,
    /// Manually complete phase.
    CompletePhase,
    /// Skip to next phase.
    Skip,
    /// Decrement timer.
    Tick,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_start_timer_from_idle() {
        let state = TimerState::Idle;
        let config = crate::TimerConfiguration::default();

        let result = StateTransitions::start(
            state,
            crate::TimerId::new(),
            &config,
            None,
        )
        .unwrap();
        assert!(matches!(result.new_state, TimerState::Working { .. }));
    }

    #[test]
    fn should_pause_running_timer() {
        let state = TimerState::Working {
            remaining_seconds: 100,
        };
        let config = crate::TimerConfiguration::default();

        let result =
            StateTransitions::pause(state, crate::TimerId::new(), &config)
                .unwrap();
        assert!(matches!(result.new_state, TimerState::Paused { .. }));
    }

    #[test]
    fn should_resume_paused_timer() {
        let working = TimerState::Working {
            remaining_seconds: 100,
        };

        let paused = TimerState::Paused {
            paused_from: Box::new(working.clone()),
            remaining_seconds: 100,
        };
        let config = crate::TimerConfiguration::default();

        let result =
            StateTransitions::resume(paused, crate::TimerId::new(), &config, None)
                .unwrap();
        assert!(matches!(result.new_state, TimerState::Working { .. }));
    }

    #[test]
    fn should_transition_from_work_to_short_break() {
        let state = TimerState::Working {
            remaining_seconds: 0,
        };
        let config = crate::TimerConfiguration::default();

        let result = StateTransitions::complete_phase(
            state,
            crate::TimerId::new(),
            &config,
            Phase::ShortBreak,
        )
        .unwrap();
        assert!(matches!(result.new_state, TimerState::ShortBreak { .. }));
    }

    #[test]
    fn should_transition_to_long_break() {
        let state = TimerState::Working {
            remaining_seconds: 0,
        };
        let config = crate::TimerConfiguration::default();

        let result = StateTransitions::complete_phase(
            state,
            crate::TimerId::new(),
            &config,
            Phase::LongBreak,
        )
        .unwrap();
        assert!(matches!(result.new_state, TimerState::LongBreak { .. }));
    }

    #[test]
    fn should_process_tick() {
        let state = TimerState::Working {
            remaining_seconds: 2,
        };
        let config = crate::TimerConfiguration::default();

        let (new_state, complete) =
            StateTransitions::tick(state, crate::TimerId::new(), &config)
                .unwrap();
        assert!(!complete);
        assert_eq!(new_state.remaining_seconds(), 1);

        let (final_state, complete) =
            StateTransitions::tick(new_state, crate::TimerId::new(), &config)
                .unwrap();
        assert!(complete);
        assert_eq!(final_state.remaining_seconds(), 0);
    }

    #[test]
    fn should_validate_transitions() {
        let idle = TimerState::Idle;

        assert!(StateTransitions::can_transition(
            &idle,
            TransitionType::Start
        ));
        assert!(!StateTransitions::can_transition(
            &idle,
            TransitionType::Pause
        ));

        let working = TimerState::Working {
            remaining_seconds: 100,
        };

        assert!(StateTransitions::can_transition(
            &working,
            TransitionType::Pause
        ));
        assert!(!StateTransitions::can_transition(
            &working,
            TransitionType::Start
        ));
    }
}
