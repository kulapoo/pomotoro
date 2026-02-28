use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskUpdatedHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskUpdatedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TaskUpdatedHandler { emitter, task_repo }
    }
}

#[async_trait]
impl EventHandler for TaskUpdatedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskUpdated>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_updated = event.as_any().downcast_ref::<domain::TaskUpdated>();

        if let Some(updated_event) = task_updated {
            // Fetch the full task from the repository
            let task = self.task_repo.get_by_id(updated_event.task_id).await?;

            if let Some(task) = task {
                self.emitter
                    .emit(domain::event_names::task::TASK_UPDATED, json!(task))
                    .map_err(|e| domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit task updated event: {e}"
                        ),
                    })?;
                self.emitter
                    .emit(domain::event_names::task::LIST_UPDATED, json!(task))
                    .map_err(|e| domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit task updated event: {e}"
                        ),
                    })?;
            } else {
                log::warn!(
                    "Task not found after update: {}",
                    updated_event.task_id
                );
            }
        }

        Ok(())
    }
}
