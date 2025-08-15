use domain::{Phase, TimerStatus, TimerState};
use domain::TaskId;

#[allow(dead_code)]
pub struct TimerStateBuilder {
    state: TimerState,
}

#[allow(dead_code)]
impl TimerStateBuilder {
    pub fn new() -> Self {
        Self {
            state: TimerState::default(),
        }
    }

    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.state.timer.phase = phase;
        self
    }

    pub fn with_status(mut self, status: TimerStatus) -> Self {
        self.state.timer.status = status;
        self
    }

    pub fn with_remaining_seconds(mut self, seconds: u32) -> Self {
        self.state.timer.remaining_seconds = seconds;
        self
    }

    pub fn with_session_count(mut self, count: u32) -> Self {
        self.state.timer.session_count = count;
        self
    }

    pub fn with_task_session_count(mut self, count: u32) -> Self {
        self.state.task_session_count = count;
        self
    }

    pub fn with_active_task(mut self, task_id: TaskId) -> Self {
        self.state.active_task_id = Some(task_id);
        self
    }

    pub fn running(mut self) -> Self {
        self.state.timer.status = TimerStatus::Running;
        self
    }

    pub fn paused(mut self) -> Self {
        self.state.timer.status = TimerStatus::Paused;
        self
    }

    pub fn stopped(mut self) -> Self {
        self.state.timer.status = TimerStatus::Stopped;
        self
    }

    pub fn work_phase(mut self) -> Self {
        self.state.timer.phase = Phase::Work;
        self.state.timer.remaining_seconds = 25 * 60; // 25 minutes
        self
    }

    pub fn short_break_phase(mut self) -> Self {
        self.state.timer.phase = Phase::ShortBreak;
        self.state.timer.remaining_seconds = 5 * 60; // 5 minutes
        self
    }

    pub fn long_break_phase(mut self) -> Self {
        self.state.timer.phase = Phase::LongBreak;
        self.state.timer.remaining_seconds = 15 * 60; // 15 minutes
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

    pub fn assert_is_running(state: &TimerState) {
        assert_eq!(state.status(), TimerStatus::Running);
    }

    pub fn assert_is_stopped(state: &TimerState) {
        assert_eq!(state.status(), TimerStatus::Stopped);
    }

    pub fn assert_is_work_phase(state: &TimerState) {
        assert_eq!(state.phase(), Phase::Work);
    }

    pub fn assert_has_active_task(state: &TimerState, task_id: TaskId) {
        assert_eq!(state.active_task_id, Some(task_id));
    }

    pub fn assert_session_count(state: &TimerState, expected: u32) {
        assert_eq!(state.session_count(), expected);
    }
}