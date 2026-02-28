use serde_json::Value;

/// Trait for emitting events. Uses JSON Value for dyn compatibility.
///
/// This trait abstracts event emission to allow different implementations
/// (e.g., Tauri, Cosmic, WebSocket, etc.) without coupling the infrastructure
/// layer to any specific UI framework.
pub trait Emitter: Send + Sync {
    fn emit(&self, event: &str, payload: Value) -> anyhow::Result<()>;
}
