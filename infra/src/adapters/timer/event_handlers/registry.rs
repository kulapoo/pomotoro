use std::{any::TypeId, sync::Arc};

use domain::{ConfigRepository, Result, TaskRepository};

use crate::adapters::events::EventSubscriber;
use crate::adapters::timer::event_handlers::TimerResetHandler;
use crate::adapters::{TimerTickService, events::app_emitter::Emitter};

use super::{
    PhaseCompletedHandler, PhaseSkippedHandler, TimerPausedHandler,
    TimerResumedHandler, TimerStartedHandler, TimerStatusChangedHandler,
    TimerTickHandler,
};

pub fn register_timer_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
    task_repo: Arc<dyn TaskRepository + Sync + Send>,
    config_repo: Arc<dyn ConfigRepository + Sync + Send>,
) -> Result<()> {
    event_bus.subscribe(Box::new(TimerTickHandler::new(
        emitter.clone(),
        timer_srv.clone(),
        config_repo.clone(),
    )))?;

    event_bus
        .subscribe(Box::new(PhaseCompletedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(PhaseSkippedHandler::new(emitter.clone())))?;
    event_bus
        .subscribe(Box::new(TimerStatusChangedHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TimerResetHandler::new(emitter.clone())))?;
    event_bus.subscribe(Box::new(TimerResumedHandler::new(
        emitter.clone(),
        timer_srv.clone(),
    )))?;
    event_bus.subscribe(Box::new(TimerPausedHandler::new(
        emitter.clone(),
        timer_srv.clone(),
    )))?;
    event_bus.subscribe(Box::new(TimerStartedHandler::new(
        emitter.clone(),
        timer_srv.clone(),
        task_repo.clone(),
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
    event_bus.clear_handlers_for_type(TypeId::of::<TimerResetHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<TimerResumedHandler>())?;
    event_bus.clear_handlers_for_type(TypeId::of::<TimerPausedHandler>())?;
    Ok(())
}
