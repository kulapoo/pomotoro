// UI Simulator Modules
pub mod app_handle;
mod register_event_handlers;
pub mod simulator;

// Re-export main components for convenience
pub use app_handle::MockAppHandle;
pub use register_event_handlers::register_test_handlers;
pub use simulator::UiSimulator;
