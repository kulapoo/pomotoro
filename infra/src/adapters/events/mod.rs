//! Events Domain Infrastructure
//!
//! Contains all event-related infrastructure implementations:
//! - Domain event bus for in-memory event handling
//! - Factory functions for creating event publishers

pub mod domain_bus;
pub mod factory;

pub use domain_bus::{DomainEventBus, EventHandler};
pub use factory::{create_event_publisher, EventPublisherArc};