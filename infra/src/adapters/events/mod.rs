//! Events Domain Infrastructure
//!
//! Contains all event-related infrastructure implementations:
//! - Domain event bus for in-memory event handling
//! - Tauri event publisher for frontend integration
//! - Composite event publisher combining multiple publishers
//! - Factory functions for creating event publishers

pub mod domain_bus;
pub mod tauri_publisher;
pub mod composite;
pub mod factory;

pub use domain_bus::*;
pub use tauri_publisher::*;
pub use composite::*;
pub use factory::*;