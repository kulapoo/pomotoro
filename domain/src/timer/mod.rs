pub mod error;
pub mod state_machine;
pub mod timer;
pub mod transitions;
pub mod events;
pub mod service;

// Re-export core types
pub use error::{Error, Result};
pub use timer::Timer;
pub use state_machine::TimerState;
pub use transitions::{TransitionResult, StateTransitions, TransitionType};
pub use service::TimerService;
// Timer-specific value objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
pub enum Status {
    Idle,
    Running,
    Paused,
    Stopped,
}

// Re-export events
pub use events::{
    Started, Paused, Reset, Tick, PhaseCompleted, PhaseSkipped,
    StatusChanged, ActiveTaskSwitched, SessionStarted, BreakSessionStarted,
    BreakSessionCompleted, WorkSessionStarted, WorkSessionCompleted, SessionFlowReset,
};