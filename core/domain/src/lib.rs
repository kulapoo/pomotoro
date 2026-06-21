pub mod audio;
pub mod config;
pub mod event_names;
pub mod shared_kernel;
pub mod task;
pub mod timer;

pub use shared_kernel::{
    EntityId, EntityMarker, Error, Event, EventPublisher, NoOpEventPublisher,
    Result, Tag, TimerConfiguration, Timestamp, duration_serde,
};

// Re-export timer configuration constants
pub use shared_kernel::value_objects::timer_configuration::{
    MAX_LONG_BREAK_DURATION, MAX_SESSIONS_UNTIL_LONG_BREAK,
    MAX_SHORT_BREAK_DURATION, MAX_WORK_DURATION, MIN_LONG_BREAK_DURATION,
    MIN_SESSIONS_UNTIL_LONG_BREAK, MIN_SHORT_BREAK_DURATION, MIN_WORK_DURATION,
};

pub use task::{
    ActiveChanged as TaskActiveChanged, Builder as TaskBuilder,
    Completed as TaskCompleted, Created as TaskCreated,
    CycleService as TaskCycleService, Id as TaskId, Marker as TaskMarker,
    Repository as TaskRepository, Reset as TaskReset, Status as TaskStatus,
    StatusChanged as TaskStatusChanged, Task, TaskDeleted, TaskPatch,
    Updated as TaskUpdated,
};

pub use timer::{
    ActiveTaskSwitched, BreakPhaseCompleted, BreakPhaseStarted,
    Error as TimerError, Paused as TimerPaused, Phase, PhaseSkipped,
    Reset as TimerReset, Result as TimerResult, SessionFlowReset,
    Started as TimerStarted, StateTransitions, Status as TimerStatus,
    StatusChanged as TimerStatusChanged, TIMER_ROW_ID, Tick as TimerTick,
    Timer, TimerRepository, TimerState, TransitionResult, TransitionType,
    WorkPhaseCompleted, WorkPhaseStarted,
};

pub use config::{
    AppearanceConfig, AudioConfig, Config, ConfigRepository, ConfigReset,
    ConfigUpdated, GeneralConfig, NotificationConfig, NotificationPosition,
    TaskCyclingBehavior, Theme,
};

pub use audio::{
    AudioAsset, AudioCategory, AudioError, AudioLibrary, AudioService,
    PlaybackHandle, PlaybackRequest,
};

// #[cfg(any(test, feature = "test-utils"))]
// pub mod test_support;
