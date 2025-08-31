// UI Simulator Modules
pub mod app_handle;
pub mod simulator;
mod register_event_handlers;

// Re-export main components for convenience
pub use app_handle::MockAppHandle;
pub use simulator::UiSimulator;
pub use register_event_handlers::register_test_handlers;