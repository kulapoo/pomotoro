use std::{any::TypeId, sync::{Arc, Mutex}};

use domain::Result;
use tauri::AppHandle;

use crate::adapters::events::EventSubscriber;

use super::{TimerStartedHandler, TimerTickHandler};

pub fn register_timer_handlers(
    event_bus: Arc<Mutex<dyn EventSubscriber + Send + Sync>>,
    app_handle: AppHandle,
) -> Result<()> {
    let mut bus = event_bus
        .lock()
        .expect("Event bus mutex was poisoned");

    bus.subscribe(Box::new(TimerStartedHandler::new(app_handle.clone())))?;
    bus.subscribe(Box::new(TimerTickHandler::new(app_handle.clone())))?;

    Ok(())
}

pub fn unregister_timer_handlers(
    event_bus: Arc<Mutex<dyn EventSubscriber + Send + Sync>>,
) -> Result<()> {
    let mut bus = event_bus
        .lock()
        .expect("Event bus mutex was poisoned");

    bus.clear_handlers_for_type(TypeId::of::<TimerStartedHandler>())?;
    bus.clear_handlers_for_type(TypeId::of::<TimerTickHandler>())?;

    Ok(())
}