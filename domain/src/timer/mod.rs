pub mod error;
pub mod events;
pub mod id;
pub mod repository;
pub mod state_machine;
pub mod timer;
pub mod transitions;

// Re-export core types
pub use error::{Error, Result};
pub use repository::TimerRepository;
pub use state_machine::TimerState;
pub use timer::{Timer, DEFAULT_TASK_ID};
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

    pub fn determine_next_break_type(current_sessions: u8, sessions_until_long_break: u8) -> Phase {
        if current_sessions % sessions_until_long_break == 0 {
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
    PhaseCompleted, PhaseSkipped, Reset, SessionFlowReset,
    Started, StatusChanged, Tick, WorkPhaseCompleted, WorkPhaseStarted,
};
