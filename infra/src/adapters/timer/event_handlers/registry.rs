use std::{any::TypeId, sync::Arc};

use domain::Result;
use tauri::AppHandle;

use crate::adapters::events::EventSubscriber;

use super::{
    PhaseCompletedHandler, PhaseSkippedHandler, TimerStartedHandler,
    TimerStatusChangedHandler, TimerTickHandler,
};

pub fn register_timer_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: AppHandle,
) -> Result<()> {
    event_bus
        .subscribe(Box::new(TimerStartedHandler::new(app_handle.clone())))?;
    event_bus.subscribe(Box::new(TimerTickHandler::new(app_handle.clone())))?;
    event_bus
        .subscribe(Box::new(PhaseCompletedHandler::new(app_handle.clone())))?;
    event_bus
        .subscribe(Box::new(PhaseSkippedHandler::new(app_handle.clone())))?;
    event_bus.subscribe(Box::new(TimerStatusChangedHandler::new(
        app_handle.clone(),
    )))?;

    Ok(())
}

pub fn unregister_timer_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
) -> Result<()> {
    event_bus.clear_handlers_for_type(TypeId::of::<TimerStartedHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<TimerTickHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<PhaseCompletedHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<PhaseSkippedHandler>())?;
    event_bus
        .clear_handlers_for_type(TypeId::of::<TimerStatusChangedHandler>())?;

    Ok(())
}
