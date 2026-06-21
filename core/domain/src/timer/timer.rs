use super::events::*;
use super::{
    Error, Phase, Result,
    state_machine::TimerState,
    transitions::{StateTransitions, TransitionType},
};
use crate::{Event, TaskId, TimerConfiguration};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// The singleton timer row's stable primary key.
///
/// There is only ever one timer in the app; this UUID is its persistent
/// row identifier. It has nothing to do with any "default" task — the
/// name is historically unfortunate.
pub static TIMER_ROW_ID: Lazy<TaskId> = Lazy::new(|| {
    TaskId::from_string("00000000-0000-0000-0000-000000000001")
        .expect("Failed to create timer row ID")
});

#[derive(Serialize, Deserialize, Clone)]
pub struct Timer {
    task_id: Option<TaskId>,
    state: TimerState,
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("task_id", &self.task_id)
            .field("state", &self.state)
            .finish()
    }
}

impl Timer {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id: Some(task_id),
            state: TimerState::new(),
        }
    }

    /// Create a timer instance with no active task.
    ///
    /// Operations that require a task (start, tick, etc.) will return
    /// [`Error::NoActiveEntity`] until a task is attached via
    /// [`Timer::set_task_id`].
    pub fn idle() -> Self {
        Self {
            task_id: None,
            state: TimerState::new(),
        }
    }

    pub fn with_state(task_id: TaskId, state: TimerState) -> Self {
        Self {
            task_id: Some(task_id),
            state,
        }
    }

    pub fn task_id(&self) -> Option<TaskId> {
        self.task_id
    }

    /// Attach a task to the timer.
    pub fn set_task_id(&mut self, task_id: TaskId) {
        self.task_id = Some(task_id);
    }

    /// Detach the task from the timer and reset state to Idle.
    ///
    /// Used when the active task is deleted. The timer cannot run again
    /// until a new task is attached.
    pub fn clear_task_id(&mut self) {
        self.task_id = None;
        self.state = TimerState::Idle;
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }

    pub fn pause_from(&self) -> Option<&TimerState> {
        match &self.state {
            TimerState::Paused { paused_from, .. } => Some(paused_from),
            _ => None,
        }
    }

    /// Returns the task_id required for state-machine transitions, or
    /// an error if no task is attached.
    fn require_task_id(&self) -> Result<TaskId> {
        self.task_id.ok_or(Error::NoActiveEntity)
    }

    pub fn start(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let task_id = self.require_task_id()?;

        if !StateTransitions::can_transition(&self.state, TransitionType::Start)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Start".to_string(),
            });
        }

        let result = StateTransitions::start(
            self.state.clone(),
            task_id,
            configuration,
        )?;

        self.state = result.new_state;

        Ok(result.events)
    }

    pub fn pause(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let task_id = self.require_task_id()?;

        if !StateTransitions::can_transition(&self.state, TransitionType::Pause)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Pause".to_string(),
            });
        }

        let result = StateTransitions::pause(
            self.state.clone(),
            task_id,
            configuration,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn resume(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let task_id = self.require_task_id()?;

        if !StateTransitions::can_transition(
            &self.state,
            TransitionType::Resume,
        ) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Resume".to_string(),
            });
        }

        let result = StateTransitions::resume(
            self.state.clone(),
            task_id,
            configuration,
        )?;

        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn reset(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        // Reset is allowed even with no task attached — it's a safe
        // "go back to Idle" operation that may emit a Reset event with
        // whichever task is currently attached (or no-op if none).
        let task_id = match self.task_id {
            Some(id) => id,
            None => {
                self.state = TimerState::Idle;
                return Ok(vec![]);
            }
        };

        let result = StateTransitions::reset(
            self.state.clone(),
            task_id,
            configuration,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn reset_phase(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let task_id = self.require_task_id()?;

        let result = StateTransitions::reset_phase(
            self.state.clone(),
            task_id,
            configuration,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn skip_phase(
        &mut self,
        configuration: &TimerConfiguration,
        next_phase: Phase,
    ) -> Result<Vec<Box<dyn Event>>> {
        let task_id = self.require_task_id()?;

        if !StateTransitions::can_transition(&self.state, TransitionType::Skip)
        {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Skip".to_string(),
            });
        }

        let result = StateTransitions::skip_phase(
            self.state.clone(),
            task_id,
            configuration,
            next_phase,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn tick(
        &mut self,
        configuration: &TimerConfiguration,
    ) -> Result<(bool, Vec<Box<dyn Event>>)> {
        let task_id = self.require_task_id()?;

        let (new_state, phase_complete) =
            StateTransitions::tick(self.state.clone(), task_id, configuration)?;
        self.state = new_state.clone();

        let mut events: Vec<Box<dyn Event>> = vec![];

        let phase = self.get_current_phase();
        let tick_event = Tick::new(
            task_id,
            phase,
            self.state.remaining_seconds(),
            1,
            configuration.clone(),
        );

        events.push(Box::new(tick_event));

        Ok((phase_complete, events))
    }

    pub fn can_start(&self) -> bool {
        self.task_id.is_some()
            && StateTransitions::can_transition(
                &self.state,
                TransitionType::Start,
            )
    }

    pub fn can_pause(&self) -> bool {
        self.task_id.is_some()
            && StateTransitions::can_transition(
                &self.state,
                TransitionType::Pause,
            )
    }

    pub fn can_resume(&self) -> bool {
        self.task_id.is_some()
            && StateTransitions::can_transition(
                &self.state,
                TransitionType::Resume,
            )
    }

    pub fn can_skip(&self) -> bool {
        self.task_id.is_some()
            && StateTransitions::can_transition(
                &self.state,
                TransitionType::Skip,
            )
    }

    pub fn remaining_seconds(
        &self,
        configuration: Option<&TimerConfiguration>,
    ) -> u32 {
        match &self.state {
            TimerState::Idle => configuration
                .map(|c| c.get_phase_duration_seconds(Phase::Work))
                .unwrap_or(25 * 60),
            _ => self.state.remaining_seconds(),
        }
    }

    pub fn set_remaining_seconds(&mut self, seconds: u32) {
        self.state = self.state.with_remaining_seconds(seconds);
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

    pub fn complete_phase(
        &mut self,
        next_phase: Phase,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let task_id = self.require_task_id()?;

        if !StateTransitions::can_transition(
            &self.state,
            TransitionType::CompletePhase,
        ) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "CompletePhase".to_string(),
            });
        }

        let result = StateTransitions::complete_phase(
            self.state.clone(),
            task_id,
            configuration,
            next_phase,
        )?;
        self.state = result.new_state;
        Ok(result.events)
    }

    pub fn start_phase(
        &mut self,
        phase: Phase,
        configuration: &TimerConfiguration,
    ) -> Result<Vec<Box<dyn Event>>> {
        let task_id = self.require_task_id()?;

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

        let events: Vec<Box<dyn Event>> =
            vec![Box::new(Started::new(task_id, phase, duration, 1))];

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskId;
    use std::time::Duration;

    fn create_test_timer() -> Timer {
        Timer::new(TaskId::new())
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
        assert!(timer.task_id().is_some());
    }

    #[test]
    fn test_idle_timer_has_no_task() {
        let timer = Timer::idle();
        assert!(timer.task_id().is_none());
        assert!(timer.is_idle());
    }

    #[test]
    fn test_start_without_task_errors() {
        let mut timer = Timer::idle();
        let config = create_test_config();
        let result = timer.start(&config);
        assert!(result.is_err());
        // NoActiveEntity maps to a timer::Error which is converted by the caller.
    }

    #[test]
    fn test_clear_task_id_resets_to_idle() {
        let mut timer = create_test_timer();
        let config = create_test_config();
        timer.start(&config).unwrap();
        assert!(timer.is_running());

        timer.clear_task_id();
        assert!(timer.task_id().is_none());
        assert!(timer.is_idle());
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

        let skip_result = timer.skip_phase(&config, Phase::ShortBreak);
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
    fn test_timer_task_id() {
        let task_id = TaskId::new();
        let timer = Timer::new(task_id);

        assert_eq!(timer.task_id(), Some(task_id));
    }
}
