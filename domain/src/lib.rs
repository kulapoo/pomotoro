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
    ActiveChanged as TaskActiveChanged, Builder as TaskBuilder,
    Completed as TaskCompleted, Created as TaskCreated,
    CycleService as TaskCycleService, Id as TaskId, Marker as TaskMarker,
    Repository as TaskRepository, Reset as TaskReset, Status as TaskStatus,
    StatusChanged as TaskStatusChanged, Task, TaskDeleted, TaskPatch,
    Updated as TaskUpdated,
};

pub use timer::{
    ActiveTaskSwitched, BreakPhaseCompleted, BreakPhaseStarted,
    DEFAULT_TASK_ID, Error as TimerError, Paused as TimerPaused, Phase,
    PhaseSkipped, Reset as TimerReset, Result as TimerResult,
    SessionFlowReset, Started as TimerStarted, StateTransitions,
    Status as TimerStatus, StatusChanged as TimerStatusChanged,
    Tick as TimerTick, Timer, TimerRepository, TimerState, TransitionResult,
    TransitionType, WorkPhaseCompleted, WorkPhaseStarted,
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
