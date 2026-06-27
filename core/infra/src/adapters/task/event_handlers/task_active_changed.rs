use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskActiveChangedHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskActiveChangedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TaskActiveChangedHandler { emitter, task_repo }
    }
}

#[async_trait]
impl EventHandler for TaskActiveChangedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskActiveChanged>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_switch =
            event.as_any().downcast_ref::<domain::TaskActiveChanged>();

        let Some(switch) = task_switch else {
            return Ok(());
        };

        let task = self.task_repo.get_by_id(switch.new_task_id).await?;

        let Some(task) = task else {
            log::warn!(
                "Task {} not found after active change; skipping emit",
                switch.new_task_id
            );
            return Ok(());
        };

        let payload = json!({
            "old_task_id": switch.old_task_id,
            "new_task_id": switch.new_task_id,
            "workflow_result": switch.workflow_result,
            "version": switch.version,
            "occurred_at": switch.occurred_at,
            "task": task,
        });

        self.emitter
            .emit(domain::event_names::task::ACTIVE_CHANGED, payload)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task active changed event: {e}"
                ),
            })?;
        Ok(())
    }
}
