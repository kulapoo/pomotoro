//! Events Domain Infrastructure
//!
//! Contains all event-related infrastructure implementations:
//! - Domain event bus for in-memory event handling with named handler support
//! - Event registrar trait for handler lifecycle management
//! - Factory functions for creating event publishers

pub mod domain_bus;
pub mod audio_events;

pub use domain_bus::{DomainEventBus, EventHandler};
