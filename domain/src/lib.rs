#![allow(clippy::module_inception)]

pub mod audio;
pub mod config;
pub mod event_names;
pub mod shared_kernel;
pub mod task;
pub mod timer;

pub use shared_kernel::{
    EntityId, EntityMarker, Error, Event, EventPublisher, NoOpEventPublisher,
    Result, Tag, TimerConfiguration, Timestamp,
    duration_serde,
};

pub use task::{
    Builder as TaskBuilder, Completed as TaskCompleted,
    Created as TaskCreated, CyclerService as TaskCyclerService,
    CyclingStrategy as TaskCyclingStrategy,
    Id as TaskId, Marker as TaskMarker, Repository as TaskRepository,
    SessionCompleted as TaskSessionCompleted,
    Status as TaskStatus, StatusChanged as TaskStatusChanged,
    SwitchWorkflowCompleted as TaskSwitchWorkflowCompleted, Task,
    Updated as TaskUpdated,
};

pub use timer::{
    ActiveTaskSwitched, BreakSessionCompleted, BreakSessionStarted,
    Error as TimerError, Paused as TimerPaused, Phase, PhaseCompleted,
    PhaseSkipped, Reset as TimerReset, Result as TimerResult, SessionFlowReset,
    Started as TimerStarted, StateTransitions,
    Status as TimerStatus, StatusChanged as TimerStatusChanged,
    Tick as TimerTick, Timer, TimerState, TransitionResult, TransitionType,
    WorkSessionCompleted, WorkSessionStarted,
};

pub use config::{
    AppearanceConfig, AudioConfig, Config, ConfigRepository, GeneralConfig,
    NotificationConfig, NotificationPosition, TaskCyclingBehavior, Theme,
};

pub use audio::{
    AudioAsset, AudioCategory, AudioError, AudioLibrary, AudioService,
    PlaybackHandle, PlaybackRequest,
};
