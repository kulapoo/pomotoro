use pomotoro_lib::timer::models::{Phase, TimerStatus, TimerState};
use std::time::Duration;
use uuid::Uuid;

pub struct TimerStateBuilder {
    state: TimerState,
}

impl TimerStateBuilder {
    pub fn new() -> Self {
        Self {
            state: TimerState::default(),
        }
    }

    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.state.phase = phase;
        self
    }

    pub fn with_status(mut self, status: TimerStatus) -> Self {
        self.state.status = status;
        self
    }

    pub fn with_remaining_seconds(mut self, seconds: u32) -> Self {
        self.state.remaining_seconds = seconds;
        self
    }

    pub fn with_session_count(mut self, count: u32) -> Self {
        self.state.session_count = count;
        self
    }

    pub fn with_task_session_count(mut self, count: u32) -> Self {
        self.state.task_session_count = count;
        self
    }

    pub fn with_active_task(mut self, task_id: Uuid) -> Self {
        self.state.active_task_id = Some(task_id);
        self
    }

    pub fn running(mut self) -> Self {
        self.state.status = TimerStatus::Running;
        self
    }

    pub fn paused(mut self) -> Self {
        self.state.status = TimerStatus::Paused;
        self
    }

    pub fn stopped(mut self) -> Self {
        self.state.status = TimerStatus::Stopped;
        self
    }

    pub fn work_phase(mut self) -> Self {
        self.state.phase = Phase::Work;
        self.state.remaining_seconds = 25 * 60; // 25 minutes
        self
    }

    pub fn short_break_phase(mut self) -> Self {
        self.state.phase = Phase::ShortBreak;
        self.state.remaining_seconds = 5 * 60; // 5 minutes
        self
    }

    pub fn long_break_phase(mut self) -> Self {
        self.state.phase = Phase::LongBreak;
        self.state.remaining_seconds = 15 * 60; // 15 minutes
        self
    }

    pub fn build(self) -> TimerState {
        self.state
    }
}

impl Default for TimerStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TimerTestAssertions;

impl TimerTestAssertions {
    pub fn assert_state_equals(actual: &TimerState, expected: &TimerState) {
        assert_eq!(actual.phase, expected.phase);
        assert_eq!(actual.status, expected.status);
        assert_eq!(actual.remaining_seconds, expected.remaining_seconds);
        assert_eq!(actual.session_count, expected.session_count);
        assert_eq!(actual.task_session_count, expected.task_session_count);
        assert_eq!(actual.active_task_id, expected.active_task_id);
    }

    pub fn assert_phase_duration(state: &TimerState, expected_duration: Duration) {
        assert_eq!(state.remaining_seconds, expected_duration.as_secs() as u32);
    }

    pub fn assert_is_work_phase(state: &TimerState) {
        assert_eq!(state.phase, Phase::Work);
    }

    pub fn assert_is_break_phase(state: &TimerState) {
        assert!(matches!(state.phase, Phase::ShortBreak | Phase::LongBreak));
    }

    pub fn assert_is_running(state: &TimerState) {
        assert_eq!(state.status, TimerStatus::Running);
    }

    pub fn assert_is_stopped(state: &TimerState) {
        assert_eq!(state.status, TimerStatus::Stopped);
    }

    pub fn assert_has_active_task(state: &TimerState, task_id: Uuid) {
        assert_eq!(state.active_task_id, Some(task_id));
    }

    pub fn assert_session_count(state: &TimerState, expected: u32) {
        assert_eq!(state.session_count, expected);
    }
}