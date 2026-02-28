//! Notification command handlers module
//!
//! This module contains all notification-related command handlers that serve as entry points
//! for external notification requests.

// Common imports and types used across notification commands
pub use domain::timer::events::{
    Paused as TimerPaused, Started as TimerStarted,
};
pub use domain::{
    Event, EventPublisher, Phase, TaskCompleted, TaskId, TimerConfiguration,
    WorkPhaseCompleted,
};
pub use std::sync::Arc;
pub use tauri::State;

// Declare submodules
mod request_notification_permission;
mod test_notification;

// Re-export all command functions
pub use request_notification_permission::request_notification_permission;
pub use test_notification::test_notification;
