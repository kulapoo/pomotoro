// Integration tests for infrastructure layer

mod core;
mod app;

// Re-export test utilities for use in test modules
pub use core::{
    context::{AppContext, AppContextBuilder},
    mocks::{MockAudioService, MockEventBus, MockTimerService, UiSimulator, UiSimulatorHandle, UiSimulatorBuilder, UiResponse},
    fixtures::{TaskFixtures, TaskBuilder, ConfigFixtures, TimerFixtures, AudioFixtures},
    database::TestDatabase,
};

