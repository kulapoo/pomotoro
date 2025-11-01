use async_trait::async_trait;
use domain::timer::events::CountdownExpired;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::EventHandler;
use domain::{Error, EventPublisher, TaskRepository, TimerRepository};

/// Event handler that triggers phase completion when countdown naturally expires
pub struct CountdownExpiredHandler {
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
}

impl CountdownExpiredHandler {
    pub fn new(event_publisher: Arc<dyn EventPublisher + Send + Sync>) -> Self {
        Self { event_publisher }
    }
}

#[async_trait]
impl EventHandler for CountdownExpiredHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<CountdownExpired>()
    }

    async fn handle(&self, event: Box<dyn domain::Event>) -> Result<(), Error> {
        let _countdown_expired = event
            .as_any()
            .downcast_ref::<CountdownExpired>()
            .ok_or_else(|| Error::RepositoryError {
                message: "Failed to downcast to CountdownExpired event"
                    .to_string(),
            })?;

        // Call the complete_timer_phase use case
        // This will handle the phase completion logic without auto-cycling

        // Auto-cycle behavior has been removed
        // The timer will now stop after each phase completes

        Ok(())
    }

    fn name(&self) -> &'static str {
        "CountdownExpiredHandler"
    }
}
