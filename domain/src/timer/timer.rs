use super::events::*;
use super::{
    Error, Phase, Result, TimerId,
    state_machine::TimerState,
    transitions::{StateTransitions, TransitionType},
};
use crate::{Event, TimerConfiguration};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Timer {
    id: TimerId,
    configuration: TimerConfiguration,
    state: TimerState,
}


impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("id", &self.id)
            .field("configuration", &self.configuration)
            .field("state", &self.state)
            .finish()
    }
}

impl Timer {
    pub fn new(id: TimerId, configuration: TimerConfiguration) -> Self {
        Self {
            id,
            configuration,
            state: TimerState::new(),
        }
    }
    
    pub fn with_state(id: TimerId, configuration: TimerConfiguration, state: TimerState) -> Self {
        Self { id, configuration, state }
    }

    pub fn id(&self) -> TimerId {
        self.id
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }

    pub fn configuration(&self) -> &TimerConfiguration {
        &self.configuration
    }

    pub fn start(&mut self) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Start)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Start".to_string(),
            });
        }

        let result = StateTransitions::start(self.state.clone(), self.id, &self.configuration)?;
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

        let result = StateTransitions::pause(self.state.clone(), self.id, &self.configuration)?;
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

        let result = StateTransitions::resume(self.state.clone(), self.id, &self.configuration)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn reset(&mut self) -> Result<Vec<Box<dyn Event>>> {
        let result = StateTransitions::reset(self.state.clone(), self.id, &self.configuration)?;
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

        let result = StateTransitions::skip_phase(self.state.clone(), self.id, &self.configuration)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn tick(&mut self) -> Result<(bool, Vec<Box<dyn Event>>)> {
        let (new_state, phase_complete) =
            StateTransitions::tick(self.state.clone(), self.id, &self.configuration)?;
        self.state = new_state.clone();

        let mut events: Vec<Box<dyn Event>> = vec![];

        let phase = self.get_current_phase();
        let tick_event = Tick::new(
            self.id,
            phase,
            self.state.remaining_seconds(),
            1,
        );
        events.push(Box::new(tick_event));

        if phase_complete {
            let next_phase = self.determine_next_phase();
            let result = StateTransitions::complete_phase(self.state.clone(), self.id, &self.configuration, next_phase)?;
            self.state = result.new_state;
            events.extend(result.events);
        }

        Ok((phase_complete, events))
    }

    fn determine_next_phase(&self) -> Phase {
        match self.state {
            TimerState::Working { .. } => Phase::ShortBreak,
            TimerState::ShortBreak { .. } | TimerState::LongBreak { .. } => Phase::Work,
            _ => Phase::Work,
        }
    }

    pub fn update_configuration(
        &mut self,
        configuration: TimerConfiguration,
    ) -> Result<()> {
        match &self.state {
            TimerState::Idle => {
                self.configuration = configuration;
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
        match &self.state {
            TimerState::Idle => self.configuration.get_phase_duration_seconds(Phase::Work),
            _ => self.state.remaining_seconds(),
        }
    }

    pub fn format_time(&self) -> String {
        let seconds = self.state.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{minutes:02}:{secs:02}")
    }

    pub fn phase_name(&self) -> &'static str {
        match &self.state {
            TimerState::Idle => "Stopped",
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
            TimerState::Idle => 0.0,
            TimerState::Working {
                remaining_seconds,
                ..
            } => {
                let total = self.configuration
                    .get_phase_duration_seconds(Phase::Work)
                    as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::ShortBreak {
                remaining_seconds,
                ..
            } => {
                let total = self.configuration
                    .get_phase_duration_seconds(Phase::ShortBreak)
                    as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::LongBreak {
                remaining_seconds,
                ..
            } => {
                let total = self.configuration
                    .get_phase_duration_seconds(Phase::LongBreak)
                    as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::Paused { .. } => 0.0,
        }
    }

    pub fn session_display(&self) -> String {
        format!("Session")
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
    
    pub fn status(&self) -> super::Status {
        self.state.status()
    }

    fn get_state_name(&self) -> String {
        match &self.state {
            TimerState::Idle => "Stopped".to_string(),
            TimerState::Working { .. } => "Working".to_string(),
            TimerState::ShortBreak { .. } => "ShortBreak".to_string(),
            TimerState::LongBreak { .. } => "LongBreak".to_string(),
            TimerState::Paused { .. } => "Paused".to_string(),
        }
    }

    pub fn get_current_phase(&self) -> Phase {
        match self.state {
            TimerState::Idle => Phase::Work,
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

    pub fn complete_phase(&mut self, next_phase: Phase) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::CompletePhase)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "CompletePhase".to_string(),
            });
        }

        let result = StateTransitions::complete_phase(self.state.clone(), self.id, &self.configuration, next_phase)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn start_phase(&mut self, phase: Phase) -> Result<Vec<Box<dyn Event>>> {
        let duration = self.configuration.get_phase_duration_seconds(phase);
        self.state = match phase {
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
        
        let events: Vec<Box<dyn Event>> = vec![
            Box::new(Started::new(
                self.id,
                phase,
                duration,
                1,
            )),
        ];
        
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_timer() -> Timer {
        let config = TimerConfiguration {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
        };
        Timer::new(TimerId::new(), config)
    }

    #[test]
    fn test_timer_creation() {
        let timer = create_test_timer();
        assert!(timer.is_idle());
        assert!(!timer.is_running());
        assert!(!timer.is_paused());
    }

    #[test]
    fn test_timer_start() {
        let mut timer = create_test_timer();

        assert!(timer.can_start());
        let result = timer.start();
        assert!(result.is_ok());

        assert!(timer.is_running());
        assert!(!timer.is_idle());
        assert!(!timer.can_start());
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = create_test_timer();

        timer.start().unwrap();
        assert!(timer.can_pause());

        let pause_result = timer.pause();
        assert!(pause_result.is_ok());
        assert!(timer.is_paused());
        assert!(!timer.can_pause());
        assert!(timer.can_resume());

        let resume_result = timer.resume();
        assert!(resume_result.is_ok());
        assert!(!timer.is_paused());
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_reset() {
        let mut timer = create_test_timer();

        timer.start().unwrap();
        assert!(timer.is_running());

        let reset_result = timer.reset();
        assert!(reset_result.is_ok());
        assert!(timer.is_idle());
        assert!(!timer.is_running());
    }

    #[test]
    fn test_timer_skip_phase() {
        let mut timer = create_test_timer();

        timer.start().unwrap();
        assert!(timer.can_skip());

        let skip_result = timer.skip_phase();
        assert!(skip_result.is_ok());
    }

    #[test]
    fn test_timer_format_time() {
        let timer = create_test_timer();
        let formatted = timer.format_time();
        assert!(formatted.contains(":"));
        assert_eq!(formatted.len(), 5);
    }

    #[test]
    fn test_timer_phase_name() {
        let mut timer = create_test_timer();

        assert_eq!(timer.phase_name(), "Stopped");

        timer.start().unwrap();
        assert_eq!(timer.phase_name(), "Focus Time");

        timer.pause().unwrap();
        assert_eq!(timer.phase_name(), "Focus Time (Paused)");
    }

    #[test]
    fn test_timer_progress_percentage() {
        let mut timer = create_test_timer();

        assert_eq!(timer.progress_percentage(), 0.0);

        timer.start().unwrap();
        let progress = timer.progress_percentage();
        assert!(progress >= 0.0);
        assert!(progress <= 100.0);
    }

    #[test]
    fn test_timer_session_display() {
        let timer = create_test_timer();
        let display = timer.session_display();
        assert_eq!(display, "Session");
    }

    #[test]
    fn test_timer_remaining_seconds() {
        let mut timer = create_test_timer();

        timer.start().unwrap();
        let remaining = timer.remaining_seconds();
        assert!(remaining > 0);
        assert_eq!(remaining, 25 * 60);
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = create_test_timer();

        timer.start().unwrap();
        let initial_remaining = timer.remaining_seconds();

        let tick_result = timer.tick();
        assert!(tick_result.is_ok());

        let (phase_complete, events) = tick_result.unwrap();
        assert!(!phase_complete);
        assert!(!events.is_empty());

        let new_remaining = timer.remaining_seconds();
        assert_eq!(new_remaining, initial_remaining - 1);
    }

    #[test]
    fn test_timer_update_configuration() {
        let mut timer = create_test_timer();

        let new_config = TimerConfiguration {
            work_duration: Duration::from_secs(30 * 60),
            short_break_duration: Duration::from_secs(10 * 60),
            long_break_duration: Duration::from_secs(20 * 60),
            sessions_until_long_break: 3,
        };

        let result = timer.update_configuration(new_config.clone());
        assert!(result.is_ok());

        assert_eq!(timer.configuration(), &new_config);
    }

    #[test]
    fn test_timer_update_configuration_while_running() {
        let mut timer = create_test_timer();
        timer.start().unwrap();

        let new_config = TimerConfiguration {
            work_duration: Duration::from_secs(30 * 60),
            short_break_duration: Duration::from_secs(10 * 60),
            long_break_duration: Duration::from_secs(20 * 60),
            sessions_until_long_break: 3,
        };

        let result = timer.update_configuration(new_config);
        assert!(result.is_err());
    }

}
