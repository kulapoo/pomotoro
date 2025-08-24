//! Events Domain Infrastructure
//!
//! Contains all event-related infrastructure implementations:
//! - In-memory event bus for event handling with thread-safe handler support
//! - Event registrar trait for handler lifecycle management
//! - Factory functions for creating event publishers

pub mod mem_event_bus;
pub mod audio_events;
pub mod app_lifecycle;
mod event_subscriber;
mod event_handler;

pub use mem_event_bus::InMemoryEventBus;
pub use event_subscriber::EventSubscriber;
pub use event_handler::EventHandler;

