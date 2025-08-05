//! Events Domain Infrastructure
//!
//! Contains all event-related infrastructure implementations:
//! - Domain event bus for in-memory event handling
//! - Tauri event publisher for frontend integration
//! - Composite event publisher combining multiple publishers
//! - Factory functions for creating event publishers

pub mod domain_bus;
pub mod publisher;
pub mod composite;
pub mod factory;

pub use domain_bus::{DomainEventBus, EventHandler};
pub use publisher::TauriEventPublisher;
pub use composite::CompositeEventPublisher;
pub use factory::{create_composite_event_publisher, create_event_publisher_with_bus, EventPublisherArc};