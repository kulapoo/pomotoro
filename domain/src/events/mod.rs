// Event constants organized by purpose and domain
//
// This module provides a clean separation between different types of events:
// - Commands: All business communication (user actions + business events)
// - UI: Technical events for frontend communication (timer:tick, etc.)
//
// All event constants are defined in the domain layer to ensure a single
// source of truth between frontend and backend.

pub mod commands;
pub mod ui;
// Backward compatibility: Re-export as domain-organized modules
// This maintains the existing API: domain::events::timer, domain::events::task, etc.
pub mod timer {
    pub use super::commands::timer::*;
    pub use super::ui::timer::*;
}

pub mod task {
    pub use super::commands::task::*;
    pub use super::ui::task::*;
}

pub mod config {
    pub use super::commands::config::*;
    pub use super::ui::config::*;
}