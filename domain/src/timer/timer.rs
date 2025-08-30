use super::events::*;
use super::{
    Error, Phase, Result,
    state_machine::TimerState,
    transitions::{StateTransitions, TransitionType},
};
use crate::{Event, TimerConfiguration};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Timer {
    state: TimerState,
}


impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("state", &self.state)
            .finish()
    }
}

impl Timer {
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self {
            state: TimerState::new(configuration),
        }
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }

    pub fn start(&mut self) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Start)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Start".to_string(),
            });
        }

        let result = StateTransitions::start(self.state.clone())?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn pause(&mut self) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Pause)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Pause".to_string(),
            });
        }

        let result = StateTransitions::pause(self.state.clone())?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn resume(&mut self) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(
            &self.state,
            TransitionType::Resume,
        ) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Resume".to_string(),
            });
        }

        let result = StateTransitions::resume(self.state.clone())?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn reset(&mut self) -> Result<Vec<Box<dyn Event>>> {
        let result = StateTransitions::reset(self.state.clone())?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn skip_phase(&mut self) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Skip)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Skip".to_string(),
            });
        }

        let result = StateTransitions::skip_phase(self.state.clone())?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn tick(&mut self) -> Result<(bool, Vec<Box<dyn Event>>)> {
        let (new_state, phase_complete) =
            StateTransitions::tick(self.state.clone())?;
        self.state = new_state.clone();

        let mut events: Vec<Box<dyn Event>> = vec![];
        
        let phase = self.get_current_phase();
        let tick_event = Tick::new(
            self.state.active_entity_id(),
            phase,
            self.state.remaining_seconds(),
            1,
        );
        events.push(Box::new(tick_event));

        if phase_complete {
            let result = StateTransitions::complete_phase(self.state.clone())?;
            self.state = result.new_state;
            events.extend(result.events);
        }

        Ok((phase_complete, events))
    }

    pub fn set_active_entity(
        &mut self,
        entity_id: Option<String>,
    ) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(
            &self.state,
            TransitionType::SwitchTask,
        ) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "SetEntity".to_string(),
            });
        }

        let result =
            StateTransitions::switch_entity(self.state.clone(), entity_id)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn update_configuration(
        &mut self,
        configuration: TimerConfiguration,
    ) -> Result<()> {
        match &self.state {
            TimerState::Idle {
                session_count,
                active_entity,
                ..
            } => {
                self.state = TimerState::Idle {
                    configuration,
                    session_count: *session_count,
                    active_entity: active_entity.clone(),
                };
                Ok(())
            }
            _ => Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "ConfigUpdate".to_string(),
            }),
        }
    }

    pub fn can_start(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Start)
    }

    pub fn can_pause(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Pause)
    }

    pub fn can_resume(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Resume)
    }

    pub fn can_skip(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Skip)
    }

    pub fn remaining_seconds(&self) -> u32 {
        self.state.remaining_seconds()
    }

    pub fn format_time(&self) -> String {
        let seconds = self.state.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{minutes:02}:{secs:02}")
    }

    pub fn phase_name(&self) -> &'static str {
        match &self.state {
            TimerState::Idle { .. } => "Stopped",
            TimerState::Working { .. } => "Focus Time",
            TimerState::ShortBreak { .. } => "Short Break",
            TimerState::LongBreak { .. } => "Long Break",
            TimerState::Paused { paused_from, .. } => {
                match paused_from.as_ref() {
                    TimerState::Working { .. } => "Focus Time (Paused)",
                    TimerState::ShortBreak { .. } => "Short Break (Paused)",
                    TimerState::LongBreak { .. } => "Long Break (Paused)",
                    _ => "Paused",
                }
            }
        }
    }

    pub fn progress_percentage(&self) -> f64 {
        match &self.state {
            TimerState::Idle { .. } => 0.0,
            TimerState::Working {
                remaining_seconds,
                configuration,
                ..
            } => {
                let total = configuration
                    .get_phase_duration_seconds(Phase::Work)
                    as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::ShortBreak {
                remaining_seconds,
                configuration,
                ..
            } => {
                let total = configuration
                    .get_phase_duration_seconds(Phase::ShortBreak)
                    as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::LongBreak {
                remaining_seconds,
                configuration,
                ..
            } => {
                let total = configuration
                    .get_phase_duration_seconds(Phase::LongBreak)
                    as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::Paused { .. } => 0.0,
        }
    }

    pub fn session_display(&self) -> String {
        let count = self.state.session_count();
        let config = self.state.configuration();
        let sessions_until_long = config.sessions_until_long_break as u32;

        let current_in_cycle = if count == 0 {
            0
        } else {
            ((count - 1) % sessions_until_long) + 1
        };

        format!("Session {current_in_cycle}/{sessions_until_long}")
    }

    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    pub fn is_paused(&self) -> bool {
        self.state.is_paused()
    }

    pub fn is_idle(&self) -> bool {
        self.state.is_idle()
    }

    fn get_state_name(&self) -> String {
        match &self.state {
            TimerState::Idle { .. } => "Stopped".to_string(),
            TimerState::Working { .. } => "Working".to_string(),
            TimerState::ShortBreak { .. } => "ShortBreak".to_string(),
            TimerState::LongBreak { .. } => "LongBreak".to_string(),
            TimerState::Paused { .. } => "Paused".to_string(),
        }
    }

    fn get_current_phase(&self) -> Phase {
        match self.state {
            TimerState::Idle { .. } => Phase::Work,
            TimerState::Working { .. } => Phase::Work,
            TimerState::ShortBreak { .. } => Phase::ShortBreak,
            TimerState::LongBreak { .. } => Phase::LongBreak,
            TimerState::Paused {
                ref paused_from, ..
            } => match paused_from.as_ref() {
                TimerState::Working { .. } => Phase::Work,
                TimerState::ShortBreak { .. } => Phase::ShortBreak,
                TimerState::LongBreak { .. } => Phase::LongBreak,
                _ => Phase::Work,
            },
        }
    }
}
