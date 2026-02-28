use async_trait::async_trait;
use domain::{
    ConfigRepository, Event, Result, TimerTick, event_names::ui_listeners,
};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::TimerTickService;
use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

pub struct TimerTickHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl TimerTickHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            emitter,
            timer_srv,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerTickHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerTick>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_tick = event
            .as_any()
            .downcast_ref::<domain::TimerTick>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to start timer tick loop".to_string(),
            })?;

        let config = self.config_repository.get_config().await?;
        let save_interval = config.general.persistence_interval_seconds;

        let phase_duration = timer_tick
            .config
            .get_phase_duration_seconds(timer_tick.phase);
        let elapsed_seconds =
            phase_duration.saturating_sub(timer_tick.remaining_seconds);

        if save_interval > 0
            && elapsed_seconds % save_interval == 0
            && elapsed_seconds > 0
        {
            let timer_srv = self.timer_srv.clone();
            println!("TIMER SAVE: {:?}", timer_srv.get_current_timer().await);
            tokio::spawn(async move {
                if let Err(e) = timer_srv.save_state().await {
                    eprintln!("Failed to save timer state: {e}");
                }
            });
        }

        self.emitter
            .emit(ui_listeners::timer::TICK, json!(timer_tick.clone()))
            .map_err(|e| domain::Error::RepositoryError {
                message: format!("Failed to emit timer tick event: {e}"),
            })?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerTickHandler"
    }
}

impl From<TimerTickHandler> for Box<dyn EventHandler> {
    fn from(handler: TimerTickHandler) -> Self {
        Box::new(handler)
    }
}
