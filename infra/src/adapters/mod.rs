//! Infrastructure Layer
//!
//! This layer contains concrete implementations of domain interfaces and handles
//! external concerns like persistence, event publishing, notifications, and
//! integration with external systems.
//!
//! ## Clean Architecture Principles
//!
//! - **Implementation**: Concrete implementations of domain abstractions
//! - **External Concerns**: Database, file system, network, UI frameworks
//! - **Dependency Direction**: Depends on domain abstractions, not implementations
//! - **Isolation**: Changes here shouldn't affect domain logic
//!
//! ## Domain-Based Structure
//!
//! - `config/`: Configuration domain infrastructure
//! - `task/`: Task domain infrastructure
//! - `timer/`: Timer domain infrastructure
//! - `audio/`: Audio domain infrastructure
//! - `events/`: Event publishing infrastructure
//! - `notifications`: System notification integration

pub mod audio;
pub mod config;
pub mod events;
pub mod notifications;
pub mod task;
pub mod timer;

// Config infrastructure
pub use config::{
    AppearanceConfigDto, AudioConfigDto, ConfigBuilder, ConfigDto, ConfigError,
    ConfigRepo, ConfigRepository, FileConfigRepo, GeneralConfigDto,
    InMemoryConfigRepository, NotificationConfigDto,
};

// Task infrastructure
pub use task::{
    FileTaskRepository, InMemoryTaskRepository, StandardTaskCyclerService,
    TaskAudioConfigDto, TaskConfigDto, TaskDto, TaskRepositoryArc,
};

// Timer infrastructure
pub use timer::TimerService;

// Audio infrastructure
pub use audio::{BG_SOUNDS, DefaultAudioAssetProvider, RodioAudioService};

// Events infrastructure
pub use events::{EventHandler, InMemoryEventBus, audio_events};

// Notifications infrastructure
pub use notifications::send_phase_notification;
