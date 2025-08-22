#![allow(clippy::module_inception)]

pub mod shared_kernel;
pub mod task;
pub mod timer;
pub mod config;
pub mod audio;
pub mod events;

pub use shared_kernel::{
    Event, EventPublisher, NoOpEventPublisher,
    Readable, Searchable, Writable,
    EntityId, EntityMarker, Tag, TimerConfiguration, Timestamp,
    Result, Error, duration_serde
};

#[cfg(any(test, feature = "test-utils"))]
pub use shared_kernel::MockEventPublisher;

pub use task::{
    Task, Builder as TaskBuilder, Id as TaskId, Marker as TaskMarker, Config as TaskConfig, Status as TaskStatus, Repository as TaskRepository,
    CyclerService as TaskCyclerService, CyclingStrategy as TaskCyclingStrategy, DefaultCyclingService as DefaultTaskCyclingService,
    Created as TaskCreated, SessionCompleted as TaskSessionCompleted, Completed as TaskCompleted, StatusChanged as TaskStatusChanged, Updated as TaskUpdated,
    SessionTransitionCompleted, SwitchWorkflowCompleted as TaskSwitchWorkflowCompleted, AutomaticCyclingCompleted as AutomaticTaskCyclingCompleted,
    CyclingExhausted as TaskCyclingExhausted
};



#[cfg(any(test, feature = "test-utils"))]
pub use task::test_repository::InMemoryRepository as InMemoryTaskRepository;
#[cfg(any(test, feature = "test-utils"))]
pub use task::test_cycling_service::TestCyclingService as TestTaskCyclingService;
#[cfg(any(test, feature = "test-utils"))]
pub use config::test_repository::InMemoryConfigRepository;

pub use timer::{
    Timer, Phase, TimerState, Status as TimerStatus,
    Error as TimerError, Result as TimerResult,
    Started as TimerStarted, Paused as TimerPaused, Reset as TimerReset, Tick as TimerTick, PhaseCompleted, PhaseSkipped,
    StatusChanged as TimerStatusChanged, ActiveTaskSwitched, SessionStarted, BreakSessionStarted,
    BreakSessionCompleted, WorkSessionStarted, WorkSessionCompleted, SessionFlowReset,
    StateTransitions, TransitionResult, TransitionType
};

pub use config::{
    Config, TaskDefaults, AudioConfig, GeneralConfig, NotificationConfig, AppearanceConfig,
    Theme, TaskCyclingBehavior, NotificationPosition, ConfigRepository
};

pub use audio::{AudioError, AudioLibrary, PlaybackRequest, PlaybackHandle, AudioAsset, AudioCategory, AudioService};