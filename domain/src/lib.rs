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

pub use task::{
    Builder as TaskBuilder, Completed as TaskCompleted, Created as TaskCreated,
    CyclerService as TaskCyclerService, Id as TaskId, Marker as TaskMarker,
    Repository as TaskRepository, SessionCompleted as TaskSessionCompleted,
    Status as TaskStatus, StatusChanged as TaskStatusChanged,
    SwitchWorkflowCompleted as TaskSwitchWorkflowCompleted, Task, TaskDeleted,
    Updated as TaskUpdated,
};

pub use timer::{
    ActiveTaskSwitched, BreakSessionCompleted, BreakSessionStarted,
    DEFAULT_TIMER_ID, Error as TimerError, Paused as TimerPaused, Phase,
    PhaseCompleted, PhaseSkipped, Reset as TimerReset, Result as TimerResult,
    SessionFlowReset, Started as TimerStarted, StateTransitions,
    Status as TimerStatus, StatusChanged as TimerStatusChanged,
    Tick as TimerTick, Timer, TimerId, TimerRepository, TimerState,
    TransitionResult, TransitionType, WorkSessionCompleted, WorkSessionStarted,
};

pub use config::{
    AppearanceConfig, AudioConfig, Config, ConfigRepository, ConfigUpdated, ConfigReset, GeneralConfig,
    NotificationConfig, NotificationPosition, TaskCyclingBehavior, Theme,
};

pub use audio::{
    AudioAsset, AudioCategory, AudioError, AudioLibrary, AudioService,
    PlaybackHandle, PlaybackRequest,
};

// #[cfg(any(test, feature = "test-utils"))]
// pub mod test_support;
