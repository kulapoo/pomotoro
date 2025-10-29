use std::{any::TypeId, sync::Arc};

use domain::{EventPublisher, Result, TaskCyclerService, TaskRepository, TimerRepository};

use crate::adapters::{events::EventSubscriber, task::event_handlers::TaskResetHandler, TimerTickService};
use crate::adapters::events::app_emitter::Emitter;

use super::{
    TaskCompletedHandler, TaskCreatedHandler, TaskDeletedHandler,
    TaskSessionCompletedHandler, TaskStatusChangedHandler,
    TaskSwitchWorkflowCompletedHandler, TaskUpdatedHandler,
};

pub fn register_task_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    emitter: Arc<dyn Emitter>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    timer_srv: Arc<TimerTickService>,
) -> Result<()> {
    event_bus.subscribe(Box::new(TaskCreatedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TaskCompletedHandler::new(
        emitter.clone(),
        cycling_service,
        task_repository.clone(),
        timer_repository,
        event_publisher,
        timer_srv.clone(),
    )))?;
    event_bus.subscribe(Box::new(TaskUpdatedHandler::new(emitter.clone(), task_repository)))?;
    event_bus.subscribe(Box::new(TaskDeletedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(TaskStatusChangedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TaskSessionCompletedHandler::new(
        emitter.clone(),
    )))?;
    event_bus.subscribe(Box::new(TaskSwitchWorkflowCompletedHandler::new(
        emitter.clone(),
    )))?;
    event_bus.subscribe(Box::new(TaskResetHandler::new(emitter.clone(), timer_srv.clone())))?;

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
    event_bus.clear_handlers_for_type(TypeId::of::<TaskResetHandler>())?;
    Ok(())
}
