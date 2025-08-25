use domain::TaskId;
use domain::{Phase, TimerConfiguration, TimerState, TimerStatus};

#[allow(dead_code)]
pub struct TimerStateBuilder {
    phase: Phase,
    status: TimerStatus,
    remaining_seconds: u32,
    session_count: u32,
    entity_session_count: u32,
    active_entity: Option<TaskId>,
    configuration: TimerConfiguration,
}

#[allow(dead_code)]
impl TimerStateBuilder {
    pub fn new() -> Self {
        Self {
            phase: Phase::Work,
            status: TimerStatus::Idle,
            remaining_seconds: 0,
            session_count: 0,
            entity_session_count: 0,
            active_entity: None,
            configuration: TimerConfiguration::default(),
        }
    }

    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_status(mut self, status: TimerStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_remaining_seconds(mut self, seconds: u32) -> Self {
        self.remaining_seconds = seconds;
        self
    }

    pub fn with_session_count(mut self, count: u32) -> Self {
        self.session_count = count;
        self
    }

    pub fn with_entity_session_count(mut self, count: u32) -> Self {
        self.entity_session_count = count;
        self
    }

    pub fn with_active_entity(mut self, task_id: TaskId) -> Self {
        self.active_entity = Some(task_id);
        self
    }

    pub fn running(mut self) -> Self {
        self.status = TimerStatus::Running;
        self
    }

    pub fn paused(mut self) -> Self {
        self.status = TimerStatus::Paused;
        self
    }

    pub fn stopped(mut self) -> Self {
        self.status = TimerStatus::Stopped;
        self
    }

    pub fn work_phase(mut self) -> Self {
        self.phase = Phase::Work;
        self.remaining_seconds = 25 * 60; // 25 minutes
        self
    }

    pub fn short_break_phase(mut self) -> Self {
        self.phase = Phase::ShortBreak;
        self.remaining_seconds = 5 * 60; // 5 minutes
        self
    }

    pub fn long_break_phase(mut self) -> Self {
        self.phase = Phase::LongBreak;
        self.remaining_seconds = 15 * 60; // 15 minutes
        self
    }

    pub fn build(self) -> TimerState {
        match (self.status, self.phase) {
            (TimerStatus::Idle, _) => TimerState::Idle {
                configuration: self.configuration,
                session_count: self.session_count,
                active_entity: self.active_entity.map(|id| id.to_string()),
            },
            (TimerStatus::Running, Phase::Work) => TimerState::Working {
                remaining_seconds: self.remaining_seconds,
                configuration: self.configuration,
                session_count: self.session_count,
                active_entity: self.active_entity.map(|id| id.to_string()),
                entity_session_count: self.entity_session_count,
            },
            (TimerStatus::Running, Phase::ShortBreak) => {
                TimerState::ShortBreak {
                    remaining_seconds: self.remaining_seconds,
                    configuration: self.configuration,
                    session_count: self.session_count,
                    active_entity: self.active_entity.map(|id| id.to_string()),
                    entity_session_count: self.entity_session_count,
                }
            }
            (TimerStatus::Running, Phase::LongBreak) => TimerState::LongBreak {
                remaining_seconds: self.remaining_seconds,
                configuration: self.configuration,
                session_count: self.session_count,
                active_entity: self.active_entity.map(|id| id.to_string()),
                entity_session_count: self.entity_session_count,
            },
            (TimerStatus::Paused, _) => {
                let base_state = match self.phase {
                    Phase::Work => TimerState::Working {
                        remaining_seconds: self.remaining_seconds,
                        configuration: self.configuration.clone(),
                        session_count: self.session_count,
                        active_entity: self
                            .active_entity
                            .map(|id| id.to_string()),
                        entity_session_count: self.entity_session_count,
                    },
                    Phase::ShortBreak => TimerState::ShortBreak {
                        remaining_seconds: self.remaining_seconds,
                        configuration: self.configuration.clone(),
                        session_count: self.session_count,
                        active_entity: self
                            .active_entity
                            .map(|id| id.to_string()),
                        entity_session_count: self.entity_session_count,
                    },
                    Phase::LongBreak => TimerState::LongBreak {
                        remaining_seconds: self.remaining_seconds,
                        configuration: self.configuration.clone(),
                        session_count: self.session_count,
                        active_entity: self
                            .active_entity
                            .map(|id| id.to_string()),
                        entity_session_count: self.entity_session_count,
                    },
                };
                TimerState::Paused {
                    paused_from: Box::new(base_state),
                    remaining_seconds: self.remaining_seconds,
                }
            }
            _ => TimerState::Idle {
                configuration: self.configuration,
                session_count: self.session_count,
                active_entity: self.active_entity.map(|id| id.to_string()),
            },
        }
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
        // Timer can be either Idle or Stopped when not running
        assert!(
            state.status() == TimerStatus::Idle
                || state.status() == TimerStatus::Stopped,
            "Expected Idle or Stopped, got {:?}",
            state.status()
        );
    }

    pub fn assert_is_work_phase(state: &TimerState) {
        assert_eq!(state.phase(), Phase::Work);
    }

    pub fn assert_has_active_task(state: &TimerState, task_id: TaskId) {
        assert_eq!(state.active_entity_id(), Some(task_id.to_string()));
    }

    pub fn assert_session_count(state: &TimerState, expected: u32) {
        assert_eq!(state.session_count(), expected);
    }
}
