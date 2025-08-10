pub mod shared_kernel;
pub mod task;
pub mod timer;
pub mod config;
pub mod audio;
pub mod events;

pub use shared_kernel::{
    DomainEvent, EventPublisher, EventSourced, NoOpEventPublisher,
    Readable, Searchable, Writable,
    EntityId, EntityMarker, Tag, TimerConfiguration, Timestamp,
    Result, Error, duration_serde
};

#[cfg(any(test, feature = "test-utils"))]
pub use shared_kernel::MockEventPublisher;

pub use task::{
    Task, TaskBuilder, TaskId, TaskMarker, TaskConfig, TaskStatus, TaskRepository,
    TaskCyclerService, TaskCyclingStrategy, DefaultTaskCyclingService,
    TaskCreated, TaskSessionCompleted, TaskCompleted, TaskStatusChanged, TaskUpdated,
    SessionTransitionCompleted, TaskSwitchWorkflowCompleted, AutomaticTaskCyclingCompleted,
    TaskCyclingExhausted
};



#[cfg(any(test, feature = "test-utils"))]
pub use task::test_repository::InMemoryTaskRepository;
#[cfg(any(test, feature = "test-utils"))]
pub use task::test_cycling_service::TestTaskCyclingService;
#[cfg(any(test, feature = "test-utils"))]
pub use config::test_repository::InMemoryConfigRepository;

pub use timer::{
    Timer, TimerId, TimerMarker, Phase, TimerState, TimerStateWithTask, TimerStatus,
    PhaseTransitionService, DefaultPhaseTransitionService, PhaseTransitionResult,
    TimerStarted, TimerPaused, TimerReset, PhaseCompleted, PhaseSkipped,
    TimerStatusChanged, ActiveTaskSwitched, SessionStarted, BreakSessionStarted,
    BreakSessionCompleted, WorkSessionStarted, WorkSessionCompleted, SessionFlowReset
};

pub use config::{
    Config, TaskDefaults, AudioConfig, GeneralConfig, NotificationConfig, AppearanceConfig,
    Theme, TaskCyclingBehavior, NotificationPosition, ConfigRepository
};

pub use audio::{AudioError, AudioLibrary, PlaybackRequest, PlaybackHandle, AudioAsset, AudioCategory, AudioService};