// Integration tests for infrastructure layer

mod app;
mod core;

// Re-export test utilities for use in test modules
pub use core::{
    context::{AppContext, AppContextBuilder},
    database::TestDatabase,
    fixtures::{
        AudioFixtures, ConfigFixtures, TaskBuilder, TaskFixtures, TimerFixtures,
    },
    mocks::{MockAudioService, UiSimulator},
    utils,
};
