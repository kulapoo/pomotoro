use super::events::*;
use super::{
    Error, Phase, Result, TimerId,
    state_machine::TimerState,
    transitions::{StateTransitions, TransitionType},
};
use crate::{Event, TimerConfiguration, TaskId};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

/// The single default timer ID used throughout the application
pub static DEFAULT_TIMER_ID: Lazy<TimerId> = Lazy::new(|| {
    TimerId::from_string("00000000-0000-0000-0000-000000000001").expect("Failed to create default timer ID")
});

#[derive(Serialize, Deserialize, Clone)]
pub struct Timer {
    id: TimerId,
    active_task_id: Option<TaskId>,
    state: TimerState,
}


impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("id", &self.id)
            .field("active_task_id", &self.active_task_id)
            .field("state", &self.state)
            .finish()
    }
}

impl Timer {
    pub fn new(id: TimerId) -> Self {
        Self {
            id,
            active_task_id: None,
            state: TimerState::new(),
        }
    }
    
    /// Create the default timer instance
    pub fn default_timer() -> Self {
        Self::new(DEFAULT_TIMER_ID.clone())
    }
    
    pub fn with_state(id: TimerId, state: TimerState) -> Self {
        Self { id, active_task_id: None, state }
    }
    
    pub fn with_active_task(mut self, task_id: TaskId) -> Self {
        self.active_task_id = Some(task_id);
        self
    }

    pub fn id(&self) -> TimerId {
        self.id
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }

    pub fn active_task_id(&self) -> Option<TaskId> {
        self.active_task_id
    }
    
    pub fn set_active_task(&mut self, task_id: TaskId) {
        self.active_task_id = Some(task_id);
    }
    
    pub fn clear_active_task(&mut self) {
        self.active_task_id = None;
    }

    pub fn start(&mut self, configuration: &TimerConfiguration) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Start)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Start".to_string(),
            });
        }

        let result = StateTransitions::start(self.state.clone(), self.id, configuration)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn pause(&mut self, configuration: &TimerConfiguration) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Pause)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Pause".to_string(),
            });
        }

        let result = StateTransitions::pause(self.state.clone(), self.id, configuration)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn resume(&mut self, configuration: &TimerConfiguration) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(
            &self.state,
            TransitionType::Resume,
        ) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Resume".to_string(),
            });
        }

        let result = StateTransitions::resume(self.state.clone(), self.id, configuration)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn reset(&mut self, configuration: &TimerConfiguration) -> Result<Vec<Box<dyn Event>>> {
        let result = StateTransitions::reset(self.state.clone(), self.id, configuration)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn skip_phase(&mut self, configuration: &TimerConfiguration) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Skip)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Skip".to_string(),
            });
        }

        let result = StateTransitions::skip_phase(self.state.clone(), self.id, configuration)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn tick(&mut self, configuration: &TimerConfiguration) -> Result<(bool, Vec<Box<dyn Event>>)> {
        let (new_state, phase_complete) =
            StateTransitions::tick(self.state.clone(), self.id, configuration)?;
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
            let result = StateTransitions::complete_phase(self.state.clone(), self.id, configuration, next_phase)?;
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

    pub fn remaining_seconds(&self, configuration: Option<&TimerConfiguration>) -> u32 {
        match &self.state {
            TimerState::Idle => {
                configuration
                    .map(|c| c.get_phase_duration_seconds(Phase::Work))
                    .unwrap_or(25 * 60)
            }
            _ => self.state.remaining_seconds(),
        }
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

    pub fn complete_phase(&mut self, next_phase: Phase, configuration: &TimerConfiguration) -> Result<Vec<Box<dyn Event>>> {
        if !StateTransitions::can_transition(&self.state, TransitionType::CompletePhase)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "CompletePhase".to_string(),
            });
        }

        let result = StateTransitions::complete_phase(self.state.clone(), self.id, configuration, next_phase)?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn start_phase(&mut self, phase: Phase, configuration: &TimerConfiguration) -> Result<Vec<Box<dyn Event>>> {
        let duration = configuration.get_phase_duration_seconds(phase);
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
    use crate::TaskId;

    fn create_test_timer() -> Timer {
        Timer::new(TimerId::new())
    }
    
    fn create_test_config() -> TimerConfiguration {
        TimerConfiguration {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
        }
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
        let config = create_test_config();

        assert!(timer.can_start());
        let result = timer.start(&config);
        assert!(result.is_ok());

        assert!(timer.is_running());
        assert!(!timer.is_idle());
        assert!(!timer.can_start());
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        timer.start(&config).unwrap();
        assert!(timer.can_pause());

        let pause_result = timer.pause(&config);
        assert!(pause_result.is_ok());
        assert!(timer.is_paused());
        assert!(!timer.can_pause());
        assert!(timer.can_resume());

        let resume_result = timer.resume(&config);
        assert!(resume_result.is_ok());
        assert!(!timer.is_paused());
        assert!(timer.is_running());
    }

    #[test]
    fn test_timer_reset() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        timer.start(&config).unwrap();
        assert!(timer.is_running());

        let reset_result = timer.reset(&config);
        assert!(reset_result.is_ok());
        assert!(timer.is_idle());
        assert!(!timer.is_running());
    }

    #[test]
    fn test_timer_skip_phase() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        timer.start(&config).unwrap();
        assert!(timer.can_skip());

        let skip_result = timer.skip_phase(&config);
        assert!(skip_result.is_ok());
    }


    #[test]
    fn test_timer_remaining_seconds() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        timer.start(&config).unwrap();
        let remaining = timer.remaining_seconds(Some(&config));
        assert!(remaining > 0);
        assert_eq!(remaining, 25 * 60);
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = create_test_timer();
        let config = create_test_config();

        timer.start(&config).unwrap();
        let initial_remaining = timer.remaining_seconds(Some(&config));

        let tick_result = timer.tick(&config);
        assert!(tick_result.is_ok());

        let (phase_complete, events) = tick_result.unwrap();
        assert!(!phase_complete);
        assert!(!events.is_empty());

        let new_remaining = timer.remaining_seconds(Some(&config));
        assert_eq!(new_remaining, initial_remaining - 1);
    }

    #[test]
    fn test_timer_active_task() {
        let mut timer = create_test_timer();
        let task_id = TaskId::new();

        assert!(timer.active_task_id().is_none());

        timer.set_active_task(task_id);
        assert_eq!(timer.active_task_id(), Some(task_id));

        timer.clear_active_task();
        assert!(timer.active_task_id().is_none());

        let timer_with_task = Timer::new(TimerId::new()).with_active_task(task_id);
        assert_eq!(timer_with_task.active_task_id(), Some(task_id));
    }

}
