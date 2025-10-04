use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, EventPublisher, Result, TaskCyclerService, TimerRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskCompletedHandler {
    emitter: Arc<dyn Emitter>,
    cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
}

impl TaskCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
        event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    ) -> Self {
        TaskCompletedHandler {
            emitter,
            cycling_service,
            timer_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl EventHandler for TaskCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_completed =
            event.as_any().downcast_ref::<domain::TaskCompleted>();

        self.emitter
            .emit(
                domain::event_names::task::LIST_UPDATED,
                json!(task_completed),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;

        // Automatically cycle to the next incomplete task
        if let Some(completed_event) = task_completed {
            let timer = self.timer_repository.get().await?;

            // Only cycle if timer is not running
            if !timer.is_running() {
                if let Some(next_task) = self
                    .cycling_service
                    .cycle_to_next_incomplete_task(Some(completed_event.task_id))
                    .await?
                {
                    // Update timer's active task
                    let mut timer = timer;
                    let previous_task_id = timer.active_task_id();
                    timer.set_active_task(next_task.id);
                    self.timer_repository.save(&timer).await?;

                    // Publish task switch event
                    let switch_event = domain::TaskSwitchWorkflowCompleted::new(
                        previous_task_id,
                        next_task.id,
                        format!("Auto-switched to task: {}", next_task.name),
                        1,
                    );
                    self.event_publisher.publish(Box::new(switch_event));
                }
            }
        }

        Ok(())
    }
}
