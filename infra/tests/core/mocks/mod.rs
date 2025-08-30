// Mock implementations for testing

mod audio_service;
mod event_bus;
mod timer_service;
mod ui_simulator;

pub use audio_service::MockAudioService;
pub use event_bus::MockEventBus;
pub use timer_service::MockTimerService;
pub use ui_simulator::{UiSimulator, UiSimulatorHandle, UiEventInterceptor, UiSimulatorBuilder, UiResponse};