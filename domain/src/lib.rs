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
    Id as TaskId, Marker as TaskMarker,
    Repository as TaskRepository, SessionCompleted as TaskSessionCompleted,
    Status as TaskStatus, StatusChanged as TaskStatusChanged,
    SwitchWorkflowCompleted as TaskSwitchWorkflowCompleted, Task, TaskDeleted,
    TaskPatch, Updated as TaskUpdated, Reset as TaskReset,
    TaskCycling, TaskCyclingExt, PureTaskCycling,
};

pub use timer::{
    ActiveTaskSwitched, BreakPhaseCompleted, BreakPhaseStarted,
    DEFAULT_TIMER_ID, Error as TimerError, Paused as TimerPaused, Phase,
    PhaseCompleted, PhaseSkipped, Reset as TimerReset, Result as TimerResult,
    SessionFlowReset, Started as TimerStarted, StateTransitions,
    Status as TimerStatus, StatusChanged as TimerStatusChanged,
    Tick as TimerTick, Timer, TimerId, TimerRepository, TimerState,
    TransitionResult, TransitionType, WorkPhaseCompleted, WorkPhaseStarted,
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
