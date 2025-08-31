// UI Simulator Modules
pub mod app_handle;
pub mod timer_actions;
pub mod task_actions;
pub mod config_actions;
pub mod audio_actions;
pub mod response;
pub mod simulator;
mod register_event_handlers;

// Re-export main components for convenience
pub use app_handle::{MockAppHandle, EmittedEvent};
pub use timer_actions::TimerUiActions;
pub use task_actions::TaskUiActions;
pub use config_actions::ConfigUiActions;
pub use audio_actions::AudioUiActions;
pub use response::UiResponse;
pub use simulator::{UiSimulator, UiSimulatorBuilder};
pub use register_event_handlers::register_test_handlers;