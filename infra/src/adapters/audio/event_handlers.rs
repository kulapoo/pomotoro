use async_trait::async_trait;
use domain::timer::events::{
    BreakSessionCompleted, BreakSessionStarted, Paused as TimerPaused,
    Started as TimerStarted, Tick as TimerTick, WorkSessionCompleted,
    WorkSessionStarted,
};
use domain::{ConfigRepository, Event, Phase, PlaybackRequest, Result};
use std::any::TypeId;
use std::sync::Arc;

use super::AudioServiceWrapper;
use crate::adapters::events::{EventHandler, EventSubscriber};

pub struct WorkSessionStartedAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl WorkSessionStartedAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for WorkSessionStartedAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<WorkSessionStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(_work_started) =
            event.as_any().downcast_ref::<WorkSessionStarted>()
        {
            let config = self.config_repository.get_config().await?;

            if config.audio.muted {
                return Ok(());
            }

            let asset_id = config
                .audio
                .work_notification_sound
                .unwrap_or_else(|| "bell".to_string());

            let request = PlaybackRequest::new(asset_id, config.audio.volume)?;

            self.audio_service.play_audio(request)?;

            if config.audio.enable_background_audio {
                if let Some(bg_sound) = config.audio.background_sound {
                    let bg_request = PlaybackRequest::new(
                        bg_sound,
                        config.audio.volume * 0.5,
                    )?
                    .with_loop()
                    .with_fade_in(2000);

                    self.audio_service.play_audio(bg_request)?;
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "WorkSessionStartedAudioHandler"
    }
}

pub struct WorkSessionCompletedAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl WorkSessionCompletedAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for WorkSessionCompletedAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<WorkSessionCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(_work_completed) =
            event.as_any().downcast_ref::<WorkSessionCompleted>()
        {
            let config = self.config_repository.get_config().await?;

            if config.audio.muted {
                return Ok(());
            }

            let asset_id = config
                .audio
                .work_notification_sound
                .unwrap_or_else(|| "chime".to_string());

            let request = PlaybackRequest::new(asset_id, config.audio.volume)?;

            self.audio_service.play_audio(request)?;

            if config.audio.enable_background_audio {
                self.audio_service.stop_all_audio()?;
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "WorkSessionCompletedAudioHandler"
    }
}

pub struct BreakSessionStartedAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl BreakSessionStartedAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for BreakSessionStartedAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<BreakSessionStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(_break_started) =
            event.as_any().downcast_ref::<BreakSessionStarted>()
        {
            let config = self.config_repository.get_config().await?;

            if config.audio.muted {
                return Ok(());
            }

            let asset_id = config
                .audio
                .break_notification_sound
                .unwrap_or_else(|| "gentle-bell".to_string());

            let request = PlaybackRequest::new(asset_id, config.audio.volume)?;

            self.audio_service.play_audio(request)?;

            if config.audio.enable_background_audio {
                if let Some(bg_sound) = config.audio.background_sound {
                    let bg_request = PlaybackRequest::new(
                        bg_sound,
                        config.audio.volume * 0.3,
                    )?
                    .with_loop()
                    .with_fade_in(2000);

                    self.audio_service.play_audio(bg_request)?;
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "BreakSessionStartedAudioHandler"
    }
}

pub struct BreakSessionCompletedAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl BreakSessionCompletedAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for BreakSessionCompletedAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<BreakSessionCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(_break_completed) =
            event.as_any().downcast_ref::<BreakSessionCompleted>()
        {
            let config = self.config_repository.get_config().await?;

            if config.audio.muted {
                return Ok(());
            }

            let asset_id = config
                .audio
                .break_notification_sound
                .unwrap_or_else(|| "ding".to_string());

            let request = PlaybackRequest::new(asset_id, config.audio.volume)?;

            self.audio_service.play_audio(request)?;

            if config.audio.enable_background_audio {
                self.audio_service.stop_all_audio()?;
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "BreakSessionCompletedAudioHandler"
    }
}

pub struct TimerStartedAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl TimerStartedAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerStartedAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_started) =
            event.as_any().downcast_ref::<TimerStarted>()
        {
            let config = self.config_repository.get_config().await?;

            if config.audio.muted {
                return Ok(());
            }

            let asset_id = match timer_started.phase {
                Phase::Work => config
                    .audio
                    .work_notification_sound
                    .unwrap_or_else(|| "bell".to_string()),
                Phase::ShortBreak | Phase::LongBreak => config
                    .audio
                    .break_notification_sound
                    .unwrap_or_else(|| "gentle-bell".to_string()),
            };

            let request = PlaybackRequest::new(asset_id, config.audio.volume)?;

            self.audio_service.play_audio(request)?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerStartedAudioHandler"
    }
}

pub struct TimerPausedAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl TimerPausedAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerPausedAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerPaused>()
    }

    async fn handle(&self, _event: Box<dyn Event>) -> Result<()> {
        let config = self.config_repository.get_config().await?;

        if config.audio.enable_background_audio {
            self.audio_service.stop_all_audio()?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerPausedAudioHandler"
    }
}

pub struct TimerTickAudioHandler {
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl TimerTickAudioHandler {
    pub fn new(
        audio_service: Arc<AudioServiceWrapper>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        Self {
            audio_service,
            config_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerTickAudioHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerTick>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_tick) = event.as_any().downcast_ref::<TimerTick>() {
            if timer_tick.remaining_seconds <= 3
                && timer_tick.remaining_seconds > 0
            {
                let config = self.config_repository.get_config().await?;

                if config.audio.muted {
                    return Ok(());
                }

                let request = PlaybackRequest::new(
                    "wooden-block".to_string(),
                    config.audio.volume * 0.5,
                )?;

                self.audio_service.play_audio(request)?;
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerTickAudioHandler"
    }
}

pub fn register_audio_event_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    audio_service: Arc<AudioServiceWrapper>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
) -> Result<()> {
    let _ = event_bus.subscribe(Box::new(WorkSessionStartedAudioHandler::new(
        audio_service.clone(),
        config_repository.clone(),
    )));

    let _ =
        event_bus.subscribe(Box::new(WorkSessionCompletedAudioHandler::new(
            audio_service.clone(),
            config_repository.clone(),
        )));

    let _ =
        event_bus.subscribe(Box::new(BreakSessionStartedAudioHandler::new(
            audio_service.clone(),
            config_repository.clone(),
        )));

    let _ =
        event_bus.subscribe(Box::new(BreakSessionCompletedAudioHandler::new(
            audio_service.clone(),
            config_repository.clone(),
        )));

    let _ = event_bus.subscribe(Box::new(TimerStartedAudioHandler::new(
        audio_service.clone(),
        config_repository.clone(),
    )));

    let _ = event_bus.subscribe(Box::new(TimerPausedAudioHandler::new(
        audio_service.clone(),
        config_repository.clone(),
    )));

    let _ = event_bus.subscribe(Box::new(TimerTickAudioHandler::new(
        audio_service.clone(),
        config_repository.clone(),
    )));

    Ok(())
}
