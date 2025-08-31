// Mock implementations for testing

mod audio_service;
mod event_bus;
mod timer_service;
pub mod ui;

pub use audio_service::MockAudioService;
pub use event_bus::MockEventBus;
pub use timer_service::MockTimerService;

// UI Simulator exports from the ui module
pub use ui::{MockAppHandle, UiSimulator};
