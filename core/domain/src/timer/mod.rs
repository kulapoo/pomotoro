pub mod error;
pub mod events;
pub mod id;
pub mod repository;
pub mod state_machine;
#[allow(clippy::module_inception)]
pub mod timer;
pub mod transitions;

// Re-export core types
pub use error::{Error, Result};
pub use repository::TimerRepository;
pub use state_machine::TimerState;
pub use timer::{ActiveTimer, TIMER_ROW_ID, Timer};
pub use transitions::{StateTransitions, TransitionResult, TransitionType};

// Timer-specific value objects
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    Hash,
)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn name(&self) -> &'static str {
        match self {
            Phase::Work => "Focus Time",
            Phase::ShortBreak => "Short Break",
            Phase::LongBreak => "Long Break",
        }
    }

    pub fn determine_next_break_type(
        current_sessions: u8,
        sessions_until_long_break: u8,
    ) -> Phase {
        // Guard against zero to avoid `is_multiple_of` panicking on a zero
        // divisor when configuration validation has been bypassed (e.g. data
        // loaded from persistence).
        let divisor = sessions_until_long_break.max(1);
        if current_sessions.is_multiple_of(divisor) {
            Phase::LongBreak
        } else {
            Phase::ShortBreak
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    Hash,
)]
pub enum Status {
    Idle,
    Running,
    Paused,
    Stopped,
}

// Re-export events
pub use events::{
    ActiveTaskSwitched, BreakPhaseCompleted, BreakPhaseStarted, Paused,
    PhaseSkipped, Reset, SessionFlowReset, Started, StatusChanged, Tick,
    WorkPhaseCompleted, WorkPhaseStarted,
};
