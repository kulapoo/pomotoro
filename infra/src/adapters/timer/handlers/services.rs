use async_trait::async_trait;
use domain::{AudioService, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Emitter;
use crate::adapters::timer::handlers::{AudioNotificationPlayer, NotificationService};
use usecases::audio::manage_library::AudioLibraryService;
use usecases::audio::notification_audio::{
    play_notification_sound, PlayNotificationSoundCmd, NotificationType,
};

pub struct ConcreteAudioNotificationPlayer {
    audio_service: Arc<Mutex<dyn AudioService>>,
    library_service: Arc<Mutex<dyn AudioLibraryService>>,
}

impl ConcreteAudioNotificationPlayer {
    pub fn new(
        audio_service: Arc<Mutex<dyn AudioService>>,
        library_service: Arc<Mutex<dyn AudioLibraryService>>,
    ) -> Self {
        Self {
            audio_service,
            library_service,
        }
    }
}

#[async_trait]
impl AudioNotificationPlayer for ConcreteAudioNotificationPlayer {
    async fn play_work_complete(&mut self) -> Result<()> {
        let cmd = PlayNotificationSoundCmd {
            notification_type: NotificationType::SessionCompleted,
            volume: Some(0.7),
        };
        
        play_notification_sound(&self.audio_service, &self.library_service, cmd).await?;
        Ok(())
    }

    async fn play_break_complete(&mut self) -> Result<()> {
        let cmd = PlayNotificationSoundCmd {
            notification_type: NotificationType::BreakCompleted,
            volume: Some(0.7),
        };
        
        play_notification_sound(&self.audio_service, &self.library_service, cmd).await?;
        Ok(())
    }
}

pub struct TauriNotificationService {
    app_handle: tauri::AppHandle,
}

impl TauriNotificationService {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl NotificationService for TauriNotificationService {
    async fn notify(&self, message: &str) -> Result<()> {
        // Use Tauri's event system to emit notification to the frontend
        self.app_handle
            .emit("notification", message)
            .map_err(|e| domain::Error::ConfigurationError {
                message: format!("Failed to emit notification: {e}"),
            })?;
            
        // TODO: Evolution paths:
        // - Add native OS notifications using tauri-plugin-notification
        // - Add toast notifications in the UI
        // - Add sound alongside visual notifications
            
        Ok(())
    }
}

#[cfg(test)]
pub struct MockNotificationService {
    pub notifications: Arc<Mutex<Vec<String>>>,
}

#[cfg(test)]
impl MockNotificationService {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[cfg(test)]
impl Default for MockNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[async_trait]
impl NotificationService for MockNotificationService {
    async fn notify(&self, message: &str) -> Result<()> {
        self.notifications.lock().await.push(message.to_string());
        Ok(())
    }
}