mod domain_event;
mod event_publisher;

pub use event_publisher::{EventPublisher, NoOpEventPublisher};

#[cfg(any(test, feature = "test-utils"))]
pub use event_publisher::MockEventPublisher;
pub use domain_event::DomainEvent;

