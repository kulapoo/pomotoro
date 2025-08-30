// Mock implementations for testing

mod audio_service;
mod event_bus;
mod repositories;
mod timer_service;

pub use audio_service::MockAudioService;
pub use event_bus::MockEventBus;
pub use repositories::{MockTaskRepository, MockConfigRepository};
pub use timer_service::MockTimerService;