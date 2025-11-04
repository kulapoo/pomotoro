//! Timer state transition logic.
//!
//! This module encapsulates all state transition rules and validation,
//! ensuring valid state changes and generating appropriate domain events.

use super::events::{
    BreakPhaseCompleted, BreakPhaseStarted, Paused, PhaseSkipped, Reset,
    Started, WorkPhaseCompleted, WorkPhaseStarted,
};
use super::state_machine::TimerState;
use super::{Error, Phase, Result};
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
        task_id: TaskId,
        configuration: &TimerConfiguration,
    ) -> Result<TransitionResult> {
        match state {
            TimerState::Idle => {
                let remaining_seconds =
                    configuration.work_duration.as_secs() as u32;
                let duration = configuration.work_duration.as_secs() as u32;

                let events: Vec<Box<dyn Event>> = vec![Box::new(Started::new(
                    task_id,
                    Phase::Work,
                    duration,
                    1,
                ))];

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
        task_id: TaskId,
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
                let phase = state.phase();

                let events: Vec<Box<dyn Event>> = vec![Box::new(Paused::new(
                    task_id,
                    phase,
                    remaining_seconds,
                    1,
                    configuration.clone(),
                ))];

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
        task_id: TaskId,
        _configuration: &TimerConfiguration,
    ) -> Result<TransitionResult> {
        match state {
            TimerState::Paused {
                paused_from,
                remaining_seconds,
            } => {
                let phase = paused_from.phase();

                // Use the remaining_seconds from the Paused state, not from paused_from
                let events: Vec<Box<dyn Event>> = vec![Box::new(Started::new(
                    task_id,
                    phase,
                    remaining_seconds,
                    1,
                ))];

                // Create the resumed state with the correct remaining seconds
                let resumed_state = match *paused_from {
                    TimerState::Working { .. } => {
                        TimerState::Working { remaining_seconds }
                    }
                    TimerState::ShortBreak { .. } => {
                        TimerState::ShortBreak { remaining_seconds }
                    }
                    TimerState::LongBreak { .. } => {
                        TimerState::LongBreak { remaining_seconds }
                    }
                    _ => {
                        return Err(Error::InvalidStateTransition {
                            from: "Paused".to_string(),
                            to: "Resume".to_string(),
                        });
                    }
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
        task_id: TaskId,
        configuration: &TimerConfiguration,
    ) -> Result<TransitionResult> {
        let events: Vec<Box<dyn Event>> = vec![Box::new(Reset::new(
            task_id,
            Phase::Work,
            1,
            configuration.clone(),
        ))];

        Ok(TransitionResult {
            new_state: TimerState::Idle,
            events,
        })
    }

    pub fn reset_phase(
        state: TimerState,
        task_id: TaskId,
        configuration: &TimerConfiguration,
    ) -> Result<TransitionResult> {
        let phase = state.phase();
        let remaining_seconds = configuration.get_phase_duration_seconds(phase);

        let new_state = match state {
            TimerState::Working { .. } => {
                TimerState::Working { remaining_seconds }
            }
            TimerState::ShortBreak { .. } => {
                TimerState::ShortBreak { remaining_seconds }
            }
            TimerState::LongBreak { .. } => {
                TimerState::LongBreak { remaining_seconds }
            }
            TimerState::Paused { paused_from, .. } => TimerState::Paused {
                remaining_seconds,
                paused_from,
            },
            TimerState::Idle => TimerState::Idle,
        };

        let events: Vec<Box<dyn Event>> = vec![Box::new(Reset::new(
            task_id,
            phase,
            1,
            configuration.clone(),
        ))];

        Ok(TransitionResult { new_state, events })
    }

    /// Completes current phase and transitions to next.
    ///
    /// Generates appropriate session completion and start events.
    pub fn complete_phase(
        state: TimerState,
        task_id: TaskId,
        configuration: &TimerConfiguration,
        next_phase: Phase,
    ) -> Result<TransitionResult> {
        let from_phase = state.phase();

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

        let mut events: Vec<Box<dyn Event>> = vec![];

        match from_phase {
            Phase::Work => {
                events.push(Box::new(WorkPhaseCompleted::new(
                    task_id,
                    configuration.get_phase_duration_seconds(Phase::Work),
                    1,
                )));
                events.push(Box::new(BreakPhaseStarted::new(
                    task_id, next_phase, duration, 1,
                )));
            }
            Phase::ShortBreak | Phase::LongBreak => {
                events.push(Box::new(BreakPhaseCompleted::new(
                    task_id,
                    from_phase,
                    configuration.get_phase_duration_seconds(from_phase),
                    1,
                )));
                if next_phase == Phase::Work {
                    events.push(Box::new(WorkPhaseStarted::new(
                        task_id, duration, 1,
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
    /// The caller must determine the next_phase based on session counting logic.
    pub fn skip_phase(
        state: TimerState,
        task_id: TaskId,
        configuration: &TimerConfiguration,
        next_phase: Phase,
    ) -> Result<TransitionResult> {
        match state {
            TimerState::Working { .. }
            | TimerState::ShortBreak { .. }
            | TimerState::LongBreak { .. } => {
                let from_phase = state.phase();

                let mut result = Self::complete_phase(
                    state.clone(),
                    task_id,
                    configuration,
                    next_phase,
                )?;

                result.events.retain(|event| {
                    let event_type = event.event_type();
                    event_type != "WorkPhaseCompleted"
                        && event_type != "BreakPhaseCompleted"
                });

                // Get the duration for the new phase
                let duration =
                    configuration.get_phase_duration_seconds(next_phase);

                // Insert PhaseSkipped and Started events at the beginning
                result.events.insert(
                    0,
                    Box::new(PhaseSkipped::new(
                        task_id, from_phase, next_phase, 1,
                    )),
                );

                result.events.insert(
                    1,
                    Box::new(Started::new(task_id, next_phase, duration, 1)),
                );

                Ok(result)
            }
            TimerState::Paused { paused_from, .. } => Self::skip_phase(
                *paused_from,
                task_id,
                configuration,
                next_phase,
            ),
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
        _task_id: TaskId,
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

impl StateTransitions {
    /// Creates a timer state for a given phase with the specified remaining seconds.
    ///
    /// This is a utility method for creating the appropriate `TimerState` variant
    /// based on a `Phase` value. It's used when we need to construct a state
    /// directly from a phase without going through the full transition logic.
    ///
    /// # Arguments
    ///
    /// * `phase` - The phase to convert into a timer state
    /// * `remaining_seconds` - The duration in seconds for this state
    ///
    /// # Returns
    ///
    /// The corresponding `TimerState` variant with the specified remaining seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use domain::timer::{Phase, TimerState, transitions::StateTransitions};
    /// let state = StateTransitions::create_state_for_phase(Phase::Work, 1500);
    /// assert!(matches!(state, TimerState::Working { remaining_seconds: 1500 }));
    /// ```
    pub fn create_state_for_phase(
        phase: Phase,
        remaining_seconds: u32,
    ) -> TimerState {
        match phase {
            Phase::Work => TimerState::Working { remaining_seconds },
            Phase::ShortBreak => TimerState::ShortBreak { remaining_seconds },
            Phase::LongBreak => TimerState::LongBreak { remaining_seconds },
        }
    }
}

/// Converts a `Phase` into a `TimerState` with zero remaining seconds.
///
/// This is useful when you need a default state for a phase, typically
/// used in testing or when initializing states without specific durations.
///
/// # Examples
///
/// ```
/// # use domain::timer::{Phase, TimerState};
/// let state: TimerState = Phase::Work.into();
/// assert!(matches!(state, TimerState::Working { remaining_seconds: 0 }));
/// ```
impl From<Phase> for TimerState {
    fn from(phase: Phase) -> Self {
        StateTransitions::create_state_for_phase(phase, 0)
    }
}

/// Converts a tuple of `(Phase, u32)` into a `TimerState`.
///
/// This provides a convenient way to create a timer state with both
/// a phase and duration in a single conversion.
///
/// # Examples
///
/// ```
/// # use domain::timer::{Phase, TimerState};
/// let state: TimerState = (Phase::ShortBreak, 300).into();
/// assert!(matches!(state, TimerState::ShortBreak { remaining_seconds: 300 }));
/// ```
impl From<(Phase, u32)> for TimerState {
    fn from((phase, remaining_seconds): (Phase, u32)) -> Self {
        StateTransitions::create_state_for_phase(phase, remaining_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_start_timer_from_idle() {
        let state = TimerState::Idle;
        let config = crate::TimerConfiguration::default();

        let result =
            StateTransitions::start(state, crate::TaskId::new(), &config)
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
            StateTransitions::pause(state, crate::TaskId::new(), &config)
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
            StateTransitions::resume(paused, crate::TaskId::new(), &config)
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
            crate::TaskId::new(),
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
            crate::TaskId::new(),
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
            StateTransitions::tick(state, crate::TaskId::new(), &config)
                .unwrap();
        assert!(!complete);
        assert_eq!(new_state.remaining_seconds(), 1);

        let (final_state, complete) =
            StateTransitions::tick(new_state, crate::TaskId::new(), &config)
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

    #[test]
    fn should_convert_phase_to_state_with_zero_seconds() {
        let work_state: TimerState = Phase::Work.into();
        assert!(matches!(
            work_state,
            TimerState::Working {
                remaining_seconds: 0
            }
        ));

        let short_break_state: TimerState = Phase::ShortBreak.into();
        assert!(matches!(
            short_break_state,
            TimerState::ShortBreak {
                remaining_seconds: 0
            }
        ));

        let long_break_state: TimerState = Phase::LongBreak.into();
        assert!(matches!(
            long_break_state,
            TimerState::LongBreak {
                remaining_seconds: 0
            }
        ));
    }

    #[test]
    fn should_convert_phase_and_duration_tuple_to_state() {
        let work_state: TimerState = (Phase::Work, 1500).into();
        assert!(matches!(
            work_state,
            TimerState::Working {
                remaining_seconds: 1500
            }
        ));

        let short_break_state: TimerState = (Phase::ShortBreak, 300).into();
        assert!(matches!(
            short_break_state,
            TimerState::ShortBreak {
                remaining_seconds: 300
            }
        ));

        let long_break_state: TimerState = (Phase::LongBreak, 900).into();
        assert!(matches!(
            long_break_state,
            TimerState::LongBreak {
                remaining_seconds: 900
            }
        ));
    }
}
