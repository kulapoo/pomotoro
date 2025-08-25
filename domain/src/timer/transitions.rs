use super::events::*;
use super::state_machine::TimerState;
use super::{Error, Phase, Result};
use crate::Event;

#[derive(Debug)]
pub struct TransitionResult {
    pub new_state: TimerState,
    pub completed_phase: Option<Phase>,
    pub work_session_completed: bool,
    pub cycle_completed: bool,
    pub events: Vec<Box<dyn Event>>,
}

pub struct StateTransitions;

impl StateTransitions {
    pub fn start(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Idle {
                configuration,
                session_count,
                active_entity,
            } => {
                if active_entity.is_none() {
                    return Err(Error::NoActiveEntity);
                }

                let remaining_seconds =
                    configuration.get_phase_duration_seconds(Phase::Work);

                let entity_id = active_entity.clone();
                let duration = configuration.work_duration.as_secs() as u32;

                let events: Vec<Box<dyn Event>> = vec![
                    Box::new(Started::new(
                        entity_id.clone(),
                        Phase::Work,
                        duration,
                        1,
                    )),
                    Box::new(WorkSessionStarted::new(
                        entity_id,
                        duration,
                        session_count,
                        1,
                        1,
                    )),
                ];

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
                    events,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{state:?}"),
                to: "Start".to_string(),
            }),
        }
    }

    pub fn pause(state: TimerState) -> Result<TransitionResult> {
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
                let entity_id = state.active_entity_id();

                let events: Vec<Box<dyn Event>> = vec![
                    Box::new(Paused::new(
                        entity_id,
                        phase,
                        remaining_seconds,
                        1,
                    )),
                ];

                Ok(TransitionResult {
                    new_state: TimerState::Paused {
                        paused_from: Box::new(state.clone()),
                        remaining_seconds,
                    },
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                    events,
                })
            }
            TimerState::Paused { .. } => Ok(TransitionResult {
                new_state: state,
                completed_phase: None,
                work_session_completed: false,
                cycle_completed: false,
                events: vec![],
            }),
            TimerState::Idle { .. } => Err(Error::InvalidStateTransition {
                from: "Stopped".to_string(),
                to: "Pause".to_string(),
            }),
        }
    }

    pub fn resume(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Paused { paused_from, .. } => {
                let phase = Self::get_phase_from_state(&paused_from);
                let entity_id = paused_from.active_entity_id();
                let remaining = paused_from.remaining_seconds();

                let events: Vec<Box<dyn Event>> = vec![
                    Box::new(Started::new(
                        entity_id, phase, remaining, 1,
                    )),
                ];

                Ok(TransitionResult {
                    new_state: *paused_from,
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                    events,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{state:?}"),
                to: "Resume".to_string(),
            }),
        }
    }

    pub fn reset(state: TimerState) -> Result<TransitionResult> {
        let configuration = state.configuration().clone();
        let active_entity = state.active_entity().map(|s| s.to_string());
        let session_count = match &state {
            TimerState::ShortBreak { session_count, .. }
            | TimerState::LongBreak { session_count, .. } => *session_count,
            _ => 0,
        };

        let events: Vec<Box<dyn Event>> = vec![
            Box::new(Reset::new(
                active_entity.clone(),
                Phase::Work,
                1,
            )),
        ];

        Ok(TransitionResult {
            new_state: TimerState::Idle {
                configuration,
                session_count,
                active_entity,
            },
            completed_phase: None,
            work_session_completed: false,
            cycle_completed: false,
            events,
        })
    }

    pub fn complete_phase(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Working {
                configuration,
                session_count,
                active_entity,
                entity_session_count,
                ..
            } => {
                let new_session_count = session_count + 1;
                let new_entity_session_count = entity_session_count + 1;

                // Clone values we need before moving them
                let entity_id = active_entity.clone();
                let work_duration =
                    configuration.work_duration.as_secs() as u32;

                let sessions_until_long =
                    configuration.sessions_until_long_break as u32;
                let (next_state, cycle_completed, to_phase, break_duration) =
                    if new_session_count % sessions_until_long == 0 {
                        let remaining_seconds = configuration
                            .get_phase_duration_seconds(Phase::LongBreak);
                        let break_dur =
                            configuration.long_break_duration.as_secs() as u32;
                        (
                            TimerState::LongBreak {
                                remaining_seconds,
                                configuration,
                                session_count: new_session_count,
                                active_entity,
                                entity_session_count: new_entity_session_count,
                            },
                            true,
                            Phase::LongBreak,
                            break_dur,
                        )
                    } else {
                        let remaining_seconds = configuration
                            .get_phase_duration_seconds(Phase::ShortBreak);
                        let break_dur =
                            configuration.short_break_duration.as_secs() as u32;
                        (
                            TimerState::ShortBreak {
                                remaining_seconds,
                                configuration,
                                session_count: new_session_count,
                                active_entity,
                                entity_session_count: new_entity_session_count,
                            },
                            false,
                            Phase::ShortBreak,
                            break_dur,
                        )
                    };

                let events: Vec<Box<dyn Event>> = vec![
                    // Phase completed event
                    Box::new(PhaseCompleted::new(
                        entity_id.clone(),
                        Phase::Work,
                        to_phase,
                        new_session_count,
                        new_session_count,
                        1,
                    )),
                    // Work session completed
                    Box::new(WorkSessionCompleted::new(
                        entity_id.clone(),
                        work_duration,
                        new_session_count,
                        1,
                        1,
                    )),
                    // Break session started
                    Box::new(BreakSessionStarted::new(
                        entity_id,
                        to_phase,
                        break_duration,
                        1,
                    )),
                ];

                Ok(TransitionResult {
                    new_state: next_state,
                    completed_phase: Some(Phase::Work),
                    work_session_completed: true,
                    cycle_completed,
                    events,
                })
            }
            TimerState::ShortBreak {
                configuration,
                session_count,
                active_entity,
                entity_session_count,
                ..
            } => {
                let remaining_seconds =
                    configuration.get_phase_duration_seconds(Phase::Work);
                let entity_id = active_entity.clone();
                let break_duration =
                    configuration.short_break_duration.as_secs() as u32;
                let work_duration =
                    configuration.work_duration.as_secs() as u32;

                let events: Vec<Box<dyn Event>> = vec![
                    // Phase completed
                    Box::new(PhaseCompleted::new(
                        entity_id.clone(),
                        Phase::ShortBreak,
                        Phase::Work,
                        session_count,
                        session_count,
                        1,
                    )),
                    // Break completed
                    Box::new(BreakSessionCompleted::new(
                        entity_id.clone(),
                        Phase::ShortBreak,
                        break_duration,
                        1,
                    )),
                    // Work session started
                    Box::new(WorkSessionStarted::new(
                        entity_id,
                        work_duration,
                        session_count,
                        1,
                        1,
                    )),
                ];

                Ok(TransitionResult {
                    new_state: TimerState::Working {
                        remaining_seconds,
                        configuration,
                        session_count,
                        active_entity,
                        entity_session_count,
                    },
                    completed_phase: Some(Phase::ShortBreak),
                    work_session_completed: false,
                    cycle_completed: false,
                    events,
                })
            }
            TimerState::LongBreak {
                configuration,
                session_count,
                active_entity,
                entity_session_count,
                ..
            } => {
                let remaining_seconds =
                    configuration.get_phase_duration_seconds(Phase::Work);
                let reset_sessions = session_count
                    >= configuration.sessions_until_long_break as u32;
                let entity_id = active_entity.clone();
                let break_duration =
                    configuration.long_break_duration.as_secs() as u32;
                let work_duration =
                    configuration.work_duration.as_secs() as u32;

                let events: Vec<Box<dyn Event>> = vec![
                    // Phase completed
                    Box::new(PhaseCompleted::new(
                        entity_id.clone(),
                        Phase::LongBreak,
                        Phase::Work,
                        session_count,
                        session_count,
                        1,
                    )),
                    // Break completed
                    Box::new(BreakSessionCompleted::new(
                        entity_id.clone(),
                        Phase::LongBreak,
                        break_duration,
                        1,
                    )),
                    // Work session started
                    Box::new(WorkSessionStarted::new(
                        entity_id,
                        work_duration,
                        if reset_sessions { 0 } else { session_count },
                        1,
                        1,
                    )),
                ];

                Ok(TransitionResult {
                    new_state: TimerState::Working {
                        remaining_seconds,
                        configuration,
                        session_count: if reset_sessions {
                            0
                        } else {
                            session_count
                        },
                        active_entity,
                        entity_session_count,
                    },
                    completed_phase: Some(Phase::LongBreak),
                    work_session_completed: false,
                    cycle_completed: false,
                    events,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{state:?}"),
                to: "CompletePhase".to_string(),
            }),
        }
    }

    pub fn skip_phase(state: TimerState) -> Result<TransitionResult> {
        match state {
            TimerState::Working { .. }
            | TimerState::ShortBreak { .. }
            | TimerState::LongBreak { .. } => {
                let mut result = Self::complete_phase(state.clone())?;

                // Replace PhaseCompleted events with PhaseSkipped events
                let from_phase = Self::get_phase_from_state(&state);
                let to_phase = Self::get_phase_from_state(&result.new_state);
                let entity_id = state.active_entity_id();

                // Filter out PhaseCompleted and WorkSessionCompleted events
                result.events.retain(|event| {
                    let event_type = event.event_type();
                    event_type != "PhaseCompleted"
                        && event_type != "WorkSessionCompleted"
                });

                result.events.insert(
                    0,
                    Box::new(PhaseSkipped::new(
                        entity_id, from_phase, to_phase, 1,
                    )),
                );

                result.completed_phase = None;
                result.work_session_completed = false;

                Ok(result)
            }
            TimerState::Paused { paused_from, .. } => {
                Self::skip_phase(*paused_from)
            }
            TimerState::Idle { .. } => Err(Error::InvalidStateTransition {
                from: "Stopped".to_string(),
                to: "Skip".to_string(),
            }),
        }
    }

    pub fn tick(mut state: TimerState) -> Result<(TimerState, bool)> {
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

    pub fn switch_entity(
        state: TimerState,
        new_entity: Option<String>,
    ) -> Result<TransitionResult> {
        match &state {
            TimerState::Idle { .. } => {
                let old_entity = state.active_entity_id();
                let phase = Phase::Work; // Idle state is always in Work phase

                let events: Vec<Box<dyn Event>> = if old_entity != new_entity {
                    vec![Box::new(ActiveTaskSwitched::new(
                        old_entity,
                        new_entity.clone(),
                        phase,
                        1,
                    ))]
                } else {
                    vec![]
                };

                let configuration = state.configuration().clone();
                let session_count = state.session_count();

                Ok(TransitionResult {
                    new_state: TimerState::Idle {
                        configuration,
                        session_count,
                        active_entity: new_entity,
                    },
                    completed_phase: None,
                    work_session_completed: false,
                    cycle_completed: false,
                    events,
                })
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{state:?}"),
                to: "SwitchEntity".to_string(),
            }),
        }
    }

    fn get_phase_from_state(state: &TimerState) -> Phase {
        match state {
            TimerState::Idle { .. } => Phase::Work,
            TimerState::Working { .. } => Phase::Work,
            TimerState::ShortBreak { .. } => Phase::ShortBreak,
            TimerState::LongBreak { .. } => Phase::LongBreak,
            TimerState::Paused { paused_from, .. } => {
                Self::get_phase_from_state(paused_from)
            }
        }
    }

    pub fn can_transition(
        from: &TimerState,
        transition: TransitionType,
    ) -> bool {
        match (from, transition) {
            (TimerState::Idle { active_entity, .. }, TransitionType::Start) => {
                active_entity.is_some()
            }
            (TimerState::Idle { .. }, TransitionType::Reset) => true,
            (TimerState::Idle { .. }, TransitionType::SwitchTask) => true,

            (TimerState::Working { .. }, TransitionType::Pause) => true,
            (TimerState::Working { .. }, TransitionType::Reset) => true,
            (TimerState::Working { .. }, TransitionType::Skip) => true,
            (
                TimerState::Working {
                    remaining_seconds, ..
                },
                TransitionType::Complete,
            ) => *remaining_seconds == 0,

            (TimerState::ShortBreak { .. }, TransitionType::Pause) => true,
            (TimerState::ShortBreak { .. }, TransitionType::Reset) => true,
            (TimerState::ShortBreak { .. }, TransitionType::Skip) => true,
            (
                TimerState::ShortBreak {
                    remaining_seconds, ..
                },
                TransitionType::Complete,
            ) => *remaining_seconds == 0,

            (TimerState::LongBreak { .. }, TransitionType::Pause) => true,
            (TimerState::LongBreak { .. }, TransitionType::Reset) => true,
            (TimerState::LongBreak { .. }, TransitionType::Skip) => true,
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

        let (final_state, complete) =
            StateTransitions::tick(new_state).unwrap();
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
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: Some(create_entity_id()),
            entity_session_count: 0,
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
