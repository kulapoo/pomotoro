use std::{any::TypeId, sync::Arc};

use domain::Result;
use tauri::AppHandle;

use crate::adapters::events::EventSubscriber;

use super::{
    TaskCompletedHandler, TaskCreatedHandler,
    TaskSessionCompletedHandler, TaskStatusChangedHandler,
    TaskSwitchWorkflowCompletedHandler, TaskUpdatedHandler,
};

pub fn register_task_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: AppHandle,
) -> Result<()> {
    event_bus
        .subscribe(Box::new(TaskCreatedHandler::new(app_handle.clone())))?;
    event_bus
        .subscribe(Box::new(TaskCompletedHandler::new(app_handle.clone())))?;
    event_bus
        .subscribe(Box::new(TaskUpdatedHandler::new(app_handle.clone())))?;
    event_bus.subscribe(Box::new(TaskStatusChangedHandler::new(
        app_handle.clone(),
    )))?;
    event_bus.subscribe(Box::new(TaskSessionCompletedHandler::new(
        app_handle.clone(),
    )))?;
    event_bus.subscribe(Box::new(TaskSwitchWorkflowCompletedHandler::new(
        app_handle.clone(),
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
