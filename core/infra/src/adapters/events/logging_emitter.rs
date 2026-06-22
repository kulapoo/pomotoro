use serde_json::Value;
use std::sync::Arc;

use crate::adapters::events::app_emitter::Emitter;

/// Decorator around [`Emitter`] that logs every client-bound event before
/// delegating to the inner emitter.
///
/// Wrap any `Emitter` with this to get visibility into all events emitted
/// to clients, regardless of which event handler produced them.
pub struct LoggingEmitter {
    inner: Arc<dyn Emitter>,
}

impl LoggingEmitter {
    pub fn new(inner: Arc<dyn Emitter>) -> Self {
        Self { inner }
    }
}

impl Emitter for LoggingEmitter {
    fn emit(&self, event: &str, payload: Value) -> anyhow::Result<()> {
        #[cfg(debug_assertions)]
        log::debug!(
            target: "infra::events::emitter",
            "emitting client event: {} | payload: {}",
            event,
            payload
        );
        self.inner.emit(event, payload)
    }
}
