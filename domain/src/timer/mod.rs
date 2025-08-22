pub mod state_machine;
pub mod timer;
pub mod transitions;
pub mod events;

// Re-export the main Timer type and related types
pub use timer::Timer;
pub use state_machine::TimerState;
pub use state_machine::TimerState as State;

// Export transition types from transitions module
pub use transitions::{TransitionResult, StateTransitions, TransitionType};

// Additional exports for backward compatibility
// Phase re-export from phase module for easier access
pub mod phase {
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
}

pub mod status {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
    pub enum Status {
        Idle,
        Running,
        Paused,
        Stopped,
    }
}

pub mod id {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Id(Uuid);
    
    impl Id {
        pub fn new() -> Self {
            Self(Uuid::new_v4())
        }
    }
    
    impl Default for Id {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl std::fmt::Display for Id {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

// Re-export Phase and Status from their modules
pub use phase::Phase;
pub use status::Status;
pub use id::Id;

// Re-export events
pub use events::{
    Started, Paused, Reset as TimerReset, Tick, PhaseCompleted, PhaseSkipped,
    StatusChanged, ActiveTaskSwitched, SessionStarted, BreakSessionStarted,
    BreakSessionCompleted, WorkSessionStarted, WorkSessionCompleted, SessionFlowReset,
};

// Alias for compatibility
pub use events::Reset;

// Create a trait for timer service compatibility
#[async_trait::async_trait]
pub trait TimerService: Send + Sync {
    async fn get_state(&self) -> crate::Result<TimerState>;
    async fn load_state(&self) -> crate::Result<()>;
    async fn switch_task(&self, task_id: crate::TaskId, task: Option<&crate::Task>) -> crate::Result<()>;
    async fn start_timer(&self, task: Option<&crate::Task>) -> crate::Result<()>;
    async fn stop_timer(&self) -> crate::Result<()>;
    async fn toggle_pause(&self) -> crate::Result<crate::TimerStatus>;
    async fn reset_current_phase(&self, task: Option<&crate::Task>) -> crate::Result<()>;
    async fn skip_to_next_phase(&self, task: Option<&crate::Task>) -> crate::Result<(Phase, Phase)>;
}

// Marker type for compatibility
pub struct Marker;