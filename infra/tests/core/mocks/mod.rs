// Mock implementations for testing

mod audio_service;
pub mod ui;

pub use audio_service::MockAudioService;

// UI Simulator exports from the ui module
pub use ui::{MockAppHandle, UiSimulator};
