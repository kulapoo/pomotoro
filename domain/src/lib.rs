#![allow(clippy::module_inception)]

pub mod audio;
pub mod config;
pub mod event_names;
pub mod shared_kernel;
pub mod task;
pub mod timer;

pub use shared_kernel::{
    EntityId, EntityMarker, Error, Event, EventPublisher, NoOpEventPublisher,
    Readable, Result, Searchable, Tag, TimerConfiguration, Timestamp, Writable,
    duration_serde,
};

#[cfg(any(test, feature = "test-utils"))]
pub use shared_kernel::MockEventPublisher;

pub use task::{
    AutomaticCyclingCompleted as AutomaticTaskCyclingCompleted,
    Builder as TaskBuilder, Completed as TaskCompleted, Config as TaskConfig,
    Created as TaskCreated, CyclerService as TaskCyclerService,
    CyclingExhausted as TaskCyclingExhausted,
    CyclingStrategy as TaskCyclingStrategy,
    DefaultCyclingService as DefaultTaskCyclingService, Id as TaskId,
    Marker as TaskMarker, Repository as TaskRepository,
    SessionCompleted as TaskSessionCompleted, SessionTransitionCompleted,
    Status as TaskStatus, StatusChanged as TaskStatusChanged,
    SwitchWorkflowCompleted as TaskSwitchWorkflowCompleted, Task,
    Updated as TaskUpdated,
};

#[cfg(any(test, feature = "test-utils"))]
pub use config::test_repository::InMemoryConfigRepository;
#[cfg(any(test, feature = "test-utils"))]
pub use task::test_cycling_service::TestCyclingService as TestTaskCyclingService;
#[cfg(any(test, feature = "test-utils"))]
pub use task::test_repository::InMemoryRepository as InMemoryTaskRepository;

pub use timer::{
    ActiveTaskSwitched, BreakSessionCompleted, BreakSessionStarted,
    Error as TimerError, Paused as TimerPaused, Phase, PhaseCompleted,
    PhaseSkipped, Reset as TimerReset, Result as TimerResult, SessionFlowReset,
    SessionStarted, Started as TimerStarted, StateTransitions,
    Status as TimerStatus, StatusChanged as TimerStatusChanged,
    Tick as TimerTick, Timer, TimerState, TransitionResult, TransitionType,
    WorkSessionCompleted, WorkSessionStarted,
};

pub use config::{
    AppearanceConfig, AudioConfig, Config, ConfigRepository, GeneralConfig,
    NotificationConfig, NotificationPosition, TaskCyclingBehavior,
    TaskDefaults, Theme,
};

pub use audio::{
    AudioAsset, AudioCategory, AudioError, AudioLibrary, AudioService,
    PlaybackHandle, PlaybackRequest,
};
