use infra::adapters::events::app_emitter::Emitter;
use serde_json::Value;
use tauri::{AppHandle, Emitter as TauriEmitter};

pub struct TauriAppHandleEmitter(AppHandle);

impl TauriAppHandleEmitter {
    pub fn new(app_handle: AppHandle) -> Self {
        Self(app_handle)
    }
}

impl Emitter for TauriAppHandleEmitter {
    fn emit(&self, event: &str, payload: Value) -> anyhow::Result<()> {
        // Use the TauriEmitter trait's emit method
        TauriEmitter::emit(&self.0, event, payload)
            .map_err(|e| anyhow::anyhow!("Failed to emit event: {}", e))
    }
}
