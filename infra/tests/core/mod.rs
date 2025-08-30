// Core test utilities for integration testing
// Organized into logical modules for maintainability

pub mod database;
pub mod fixtures;
pub mod context;

// Re-export commonly used items for convenience
pub use database::{TestDatabase, IsolatedDb};

pub use fixtures::{
    TaskFixtures, TaskBuilder,
    ConfigFixtures,
    TimerFixtures,
    AudioFixtures,
};

// pub use mocks::{
//     MockAudioService,
//     MockEventBus,
//     MockTaskRepository,
//     MockConfigRepository,
//     MockTimerService,
// };

pub use context::{AppContext, AppContextBuilder};

