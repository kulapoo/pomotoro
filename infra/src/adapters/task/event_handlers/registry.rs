use std::{any::TypeId, sync::Arc};

use domain::Result;

use crate::adapters::events::EventSubscriber;
use crate::adapters::events::app_emitter::Emitter;

use super::{
    TaskCompletedHandler, TaskCreatedHandler, TaskDeletedHandler,
    TaskSessionCompletedHandler, TaskStatusChangedHandler,
    TaskSwitchWorkflowCompletedHandler, TaskUpdatedHandler,
};

pub fn register_task_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    emitter: Arc<dyn Emitter>,
) -> Result<()> {
    event_bus.subscribe(Box::new(TaskCreatedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(TaskCompletedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TaskUpdatedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TaskDeletedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(TaskStatusChangedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TaskSessionCompletedHandler::new(
        emitter.clone(),
    )))?;
    event_bus.subscribe(Box::new(TaskSwitchWorkflowCompletedHandler::new(
        emitter.clone(),
    )))?;

    Ok(())
}

pub fn unregister_task_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
) -> Result<()> {
    event_bus.clear_handlers_for_type(TypeId::of::<TaskCreatedHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<TaskCompletedHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<TaskUpdatedHandler>())?;
    event_bus
        .clear_handlers_for_type(TypeId::of::<TaskStatusChangedHandler>())?;
    event_bus
        .clear_handlers_for_type(TypeId::of::<TaskSessionCompletedHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<
        TaskSwitchWorkflowCompletedHandler,
    >())?;

    Ok(())
}
