//! Events Domain Infrastructure
//!
//! Contains all event-related infrastructure implementations:
//! - In-memory event bus for event handling with thread-safe handler support
//! - Event registrar trait for handler lifecycle management
//! - Factory functions for creating event publishers

pub mod app_lifecycle;
pub mod audio_events;
mod event_handler;
mod event_subscriber;
pub mod mem_event_bus;
pub mod app_emitter;

pub use event_handler::EventHandler;
pub use event_subscriber::EventSubscriber;
pub use mem_event_bus::InMemoryEventBus;
