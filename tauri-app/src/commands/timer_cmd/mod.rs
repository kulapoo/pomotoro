//! Timer command handlers module
//!
//! This module contains all timer-related command handlers that serve as entry points
//! for external timer control requests.

// Common imports and types used across multiple timer commands
pub use infra::adapters::events::mem_event_bus::EventPublisherArc;
pub use infra::adapters::TimerRepositoryArc;
pub use anyhow::Context;
pub use domain::{event_names::ui_listeners, Task, TaskRepository, Timer, TimerStatus};
pub use log::{debug, info};
pub use std::sync::Arc;
pub use tauri::{AppHandle, Emitter, State};

// Declare submodules
mod get_timer_state;
mod pause_timer;
mod reset_timer;
mod resume_timer;
mod skip_phase;
mod start_timer;
mod switch_active_task;
mod update_timer_secs;

// Re-export all command functions
pub use get_timer_state::get_timer_state;
pub use pause_timer::pause_timer;
pub use reset_timer::reset_timer;
pub use resume_timer::resume_timer;
pub use skip_phase::skip_phase;
pub use start_timer::start_timer;
pub use switch_active_task::switch_active_task;
pub use update_timer_secs::update_timer_secs;
