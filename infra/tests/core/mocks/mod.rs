// Mock implementations for testing

mod audio_service;
mod timer_service;
pub mod ui;

pub use audio_service::MockAudioService;
pub use timer_service::MockTimerService;

// UI Simulator exports from the ui module
pub use ui::{MockAppHandle, UiSimulator};
