pub mod error;
pub mod events;
pub mod service;
pub mod state_machine;
pub mod timer;
pub mod transitions;

// Re-export core types
pub use error::{Error, Result};
pub use service::TimerService;
pub use state_machine::TimerState;
pub use timer::Timer;
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
    ActiveTaskSwitched, BreakSessionCompleted, BreakSessionStarted, Paused,
    PhaseCompleted, PhaseSkipped, Reset, SessionFlowReset, SessionStarted,
    Started, StatusChanged, Tick, WorkSessionCompleted, WorkSessionStarted,
};
