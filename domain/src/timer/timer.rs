use super::events::*;
use super::{
    Error, Phase, Result,
    state_machine::TimerState,
    transitions::{StateTransitions, TransitionResult, TransitionType},
};
use crate::{EventPublisher, TimerConfiguration};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Timer {
    state: TimerState,
    #[serde(skip)]
    event_publisher: Option<Box<dyn EventPublisher>>,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            state: TimerState::default(),
            event_publisher: None,
        }
    }
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("state", &self.state)
            .field("has_publisher", &self.event_publisher.is_some())
            .finish()
    }
}

impl Timer {
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self {
            state: TimerState::new(configuration),
            event_publisher: None,
        }
    }

    pub fn with_event_publisher(mut self, publisher: Box<dyn EventPublisher>) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }

    pub fn start(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Start) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Start".to_string(),
            });
        }

        let result = StateTransitions::start(self.state.clone())?;
        self.publish_events(result)
    }

    pub fn pause(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Pause) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Pause".to_string(),
            });
        }

        let result = StateTransitions::pause(self.state.clone())?;
        self.publish_events(result)
    }

    pub fn resume(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Resume) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Resume".to_string(),
            });
        }

        let result = StateTransitions::resume(self.state.clone())?;
        self.publish_events(result)
    }

    pub fn reset(&mut self) -> Result<()> {
        let result = StateTransitions::reset(self.state.clone())?;
        self.publish_events(result)
    }

    pub fn skip_phase(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Skip) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Skip".to_string(),
            });
        }

        let result = StateTransitions::skip_phase(self.state.clone())?;
        self.publish_events(result)
    }

    pub fn tick(&mut self) -> Result<bool> {
        let (new_state, phase_complete) = StateTransitions::tick(self.state.clone())?;
        let _old_state = self.state.clone();
        self.state = new_state.clone();

        if let Some(publisher) = &self.event_publisher {
            let phase = self.get_current_phase();
            let event = Tick::new(
                self.state.active_entity_id(),
                phase,
                self.state.remaining_seconds(),
                1,
            );
            publisher.publish(Box::new(event));
        }

        if phase_complete {
            let result = StateTransitions::complete_phase(self.state.clone())?;
            self.publish_events(result)?;
        }

        Ok(phase_complete)
    }

    pub fn set_active_entity(&mut self, entity_id: Option<String>) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::SwitchTask) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "SetEntity".to_string(),
            });
        }

        let result = StateTransitions::switch_entity(self.state.clone(), entity_id)?;
        self.publish_events(result)
    }

    pub fn update_configuration(&mut self, configuration: TimerConfiguration) -> Result<()> {
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
        format!("{:02}:{:02}", minutes, secs)
    }

    pub fn phase_name(&self) -> &'static str {
        match &self.state {
            TimerState::Idle { .. } => "Stopped",
            TimerState::Working { .. } => "Focus Time",
            TimerState::ShortBreak { .. } => "Short Break",
            TimerState::LongBreak { .. } => "Long Break",
            TimerState::Paused { paused_from, .. } => match paused_from.as_ref() {
                TimerState::Working { .. } => "Focus Time (Paused)",
                TimerState::ShortBreak { .. } => "Short Break (Paused)",
                TimerState::LongBreak { .. } => "Long Break (Paused)",
                _ => "Paused",
            },
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
                let total = configuration.get_phase_duration_seconds(Phase::Work) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::ShortBreak {
                remaining_seconds,
                configuration,
                ..
            } => {
                let total = configuration.get_phase_duration_seconds(Phase::ShortBreak) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::LongBreak {
                remaining_seconds,
                configuration,
                ..
            } => {
                let total = configuration.get_phase_duration_seconds(Phase::LongBreak) as f64;
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

        format!("Session {}/{}", current_in_cycle, sessions_until_long)
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

    fn publish_events(&mut self, result: TransitionResult) -> Result<()> {
        self.state = result.new_state;

        if let Some(publisher) = &self.event_publisher {
            for event in result.events {
                publisher.publish(event);
            }
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_kernel::MockEventPublisher;

    fn create_entity_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    #[test]
    fn should_create_default_timer() {
        let timer = Timer::default();
        assert!(timer.is_idle());
        assert_eq!(timer.remaining_seconds(), 1500);
    }

    #[test]
    fn should_start_timer_with_entity() {
        let mut timer = Timer::default();
        let entity_id = create_entity_id();

        timer.set_active_entity(Some(entity_id)).unwrap();
        timer.start().unwrap();

        assert!(timer.is_running());
        assert_eq!(timer.phase_name(), "Focus Time");
    }

    #[test]
    fn should_not_start_without_entity() {
        let mut timer = Timer::default();
        let result = timer.start();
        assert!(result.is_err());
    }

    #[test]
    fn should_pause_and_resume() {
        let mut timer = Timer::default();
        let entity_id = create_entity_id();

        timer.set_active_entity(Some(entity_id)).unwrap();
        timer.start().unwrap();

        let initial_time = timer.remaining_seconds();
        timer.pause().unwrap();
        assert!(timer.is_paused());

        timer.resume().unwrap();
        assert!(timer.is_running());
        assert_eq!(timer.remaining_seconds(), initial_time);
    }

    #[test]
    fn should_process_ticks() {
        let mut timer = Timer::default();
        let entity_id = create_entity_id();

        timer.set_active_entity(Some(entity_id)).unwrap();
        timer.start().unwrap();

        let initial_time = timer.remaining_seconds();
        let complete = timer.tick().unwrap();
        assert!(!complete);
        assert_eq!(timer.remaining_seconds(), initial_time - 1);
    }

    #[test]
    fn should_skip_phase() {
        let mut timer = Timer::default();
        let entity_id = create_entity_id();

        timer.set_active_entity(Some(entity_id)).unwrap();
        timer.start().unwrap();

        timer.skip_phase().unwrap();
        assert!(timer.state.is_break_phase());
    }

    #[test]
    fn should_publish_events() {
        let publisher = MockEventPublisher::new();

        let mut timer = Timer::default().with_event_publisher(Box::new(publisher));

        let entity_id = create_entity_id();
        timer.set_active_entity(Some(entity_id)).unwrap();
        timer.start().unwrap();
    }

    #[test]
    fn should_format_time_correctly() {
        let timer = Timer::new(TimerConfiguration::default());
        assert_eq!(timer.format_time(), "25:00");

        let mut timer = Timer::default();
        let entity_id = create_entity_id();
        timer.set_active_entity(Some(entity_id)).unwrap();
        timer.start().unwrap();
        assert_eq!(timer.format_time(), "25:00");
    }

    #[test]
    fn should_calculate_progress() {
        let mut timer = Timer::default();
        let entity_id = create_entity_id();

        timer.set_active_entity(Some(entity_id)).unwrap();
        timer.start().unwrap();

        assert_eq!(timer.progress_percentage(), 0.0);

        for _ in 0..750 {
            timer.tick().unwrap();
        }
        let progress = timer.progress_percentage();
        assert!((progress - 50.0).abs() < 1.0);
    }
}
