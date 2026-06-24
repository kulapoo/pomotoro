use std::{any::TypeId, sync::Arc};

use domain::{Result, TaskRepository};

use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{
    TimerTickService, events::EventSubscriber,
    task::event_handlers::TaskResetHandler,
};

use super::{
    TaskActiveChangedHandler, TaskCompletedHandler, TaskCreatedHandler,
    TaskDeletedHandler, TaskStatusChangedHandler, TaskUpdatedHandler,
};

pub fn register_task_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    emitter: Arc<dyn Emitter>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    timer_srv: Arc<TimerTickService>,
) -> Result<()> {
    event_bus.subscribe(Box::new(TaskCreatedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(TaskCompletedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TaskUpdatedHandler::new(
        emitter.clone(),
        task_repository,
    )))?;
    event_bus.subscribe(Box::new(TaskDeletedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(TaskStatusChangedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(TaskActiveChangedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TaskResetHandler::new(
        emitter.clone(),
        timer_srv.clone(),
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
        .clear_handlers_for_type(TypeId::of::<TaskActiveChangedHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<TaskResetHandler>())?;
    Ok(())
}
