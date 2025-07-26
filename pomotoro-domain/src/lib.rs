pub mod shared_kernel;
pub mod task;
pub mod timer;
pub mod config;
pub mod audio;
pub mod events;

// Re-export core shared types and traits
pub use shared_kernel::{
    DomainEvent, EventPublisher, EventSourced, NoOpEventPublisher,
    Readable, Searchable, Writable,
    EntityId, EntityMarker, Tag, TimerConfiguration, Timestamp,
    Result, Error, duration_serde
};

// Re-export task domain
pub use task::{
    Task, TaskBuilder, TaskId, TaskMarker, TaskConfig, TaskStatus, TaskRepository,
    DefaultTaskCyclingService, TaskCyclingService, TaskCyclingStrategy,
    TaskCreated, TaskSessionCompleted, TaskCompleted, TaskStatusChanged, TaskUpdated,
    SessionTransitionCompleted, TaskSwitchWorkflowCompleted, AutomaticTaskCyclingCompleted,
    TaskCyclingExhausted
};

// Re-export timer domain
pub use timer::{
    Timer, TimerId, TimerMarker, Phase, TimerState, TimerStateWithTask, TimerStatus,
    PhaseTransitionService, DefaultPhaseTransitionService, PhaseTransitionResult,
    TimerStarted, TimerPaused, TimerReset, PhaseCompleted, PhaseSkipped, 
    TimerStatusChanged, ActiveTaskSwitched, SessionStarted, BreakSessionStarted,
    BreakSessionCompleted, WorkSessionStarted, WorkSessionCompleted, SessionFlowReset
};

// Re-export config domain
pub use config::*;

// Re-export audio domain
pub use audio::*;