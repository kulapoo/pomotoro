use std::{any::TypeId, sync::Arc};

use domain::Result;

use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::events::EventSubscriber;

use super::{
    PhaseCompletedHandler, PhaseSkippedHandler, TimerStartedHandler,
    TimerStatusChangedHandler, TimerTickHandler,
};

pub fn register_timer_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    emitter: Arc<dyn Emitter>,
) -> Result<()> {
    event_bus
        .subscribe(Box::new(TimerStartedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TimerTickHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(PhaseCompletedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(PhaseSkippedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TimerStatusChangedHandler::new(
        emitter.clone(),
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
