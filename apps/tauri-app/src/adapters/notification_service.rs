use async_trait::async_trait;
use infra::adapters::notifications::{
    NotificationEvent, NotificationServiceTrait,
};
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_notification::{NotificationExt, PermissionState};
use tokio::sync::RwLock;

pub struct NotificationService {
    app_handle: AppHandle,
    config: Arc<RwLock<domain::NotificationConfig>>,
    permission_granted: Arc<RwLock<Option<bool>>>,
}

impl NotificationService {
    pub fn new(
        app_handle: AppHandle,
        config: domain::NotificationConfig,
    ) -> Self {
        Self {
            app_handle,
            config: Arc::new(RwLock::new(config)),
            permission_granted: Arc::new(RwLock::new(None)),
        }
    }

    async fn should_send_notification(
        &self,
        event: &NotificationEvent,
    ) -> bool {
        let config = self.config.read().await;

        if !config.enable_desktop_notifications {
            return false;
        }

        match event {
            NotificationEvent::WorkPhaseCompleted { .. }
            | NotificationEvent::BreakStarted { .. }
            | NotificationEvent::BreakCompleted { .. } => {
                config.show_phase_transition_notifications
            }
            NotificationEvent::TaskCompleted { .. } => {
                config.show_task_completion_notifications
            }
            NotificationEvent::SessionStarted { .. }
            | NotificationEvent::TimerPaused { .. }
            | NotificationEvent::TimerResumed { .. } => true,
        }
    }

    async fn ensure_permission(&self) -> domain::Result<bool> {
        let mut cached = self.permission_granted.write().await;

        if let Some(granted) = *cached {
            return Ok(granted);
        }

        let permission = match self.app_handle.notification().permission_state()
        {
            Ok(state) => match state {
                PermissionState::Granted => true,
                PermissionState::Denied => false,
                PermissionState::Prompt
                | PermissionState::PromptWithRationale => {
                    matches!(
                        self.app_handle.notification().request_permission(),
                        Ok(PermissionState::Granted)
                    )
                }
            },
            Err(_) => false,
        };

        *cached = Some(permission);
        Ok(permission)
    }
}

#[async_trait]
impl NotificationServiceTrait for NotificationService {
    async fn send_notification(
        &self,
        event: NotificationEvent,
    ) -> domain::Result<()> {
        if !self.should_send_notification(&event).await {
            return Ok(());
        }

        let config = self.config.read().await;
        let send_desktop = config.enable_desktop_notifications;
        let _send_sound = config.enable_sound_notifications;
        drop(config);

        if send_desktop {
            let has_permission = self.ensure_permission().await?;
            if has_permission {
                let context = event.to_context();

                #[cfg(target_os = "linux")]
                {
                    let mut notification = notify_rust::Notification::new();
                    notification
                        .appname("Pomotoro")
                        .summary(&context.title)
                        .body(&context.body);
                    if let Some(icon) = &context.icon {
                        notification.icon(icon);
                    } else {
                        notification.auto_icon();
                    }
                    if let Err(e) = notification.show() {
                        log::warn!("Failed to show notification: {}", e);
                    }
                }

                #[cfg(not(target_os = "linux"))]
                {
                    let mut builder = self
                        .app_handle
                        .notification()
                        .builder()
                        .title(&context.title)
                        .body(&context.body);

                    if let Some(icon) = context.icon {
                        builder = builder.icon(icon);
                    }

                    match builder.show() {
                        Ok(_) => {}
                        Err(e) => {
                            log::warn!("Failed to show notification: {}", e);
                        }
                    }
                }
            }
        }

        // Sound notifications will be handled separately through the audio service
        // when the notification events are published

        Ok(())
    }

    async fn request_permission(&self) -> domain::Result<bool> {
        self.ensure_permission().await
    }

    async fn check_permission(&self) -> domain::Result<bool> {
        let cached = self.permission_granted.read().await;
        if let Some(granted) = *cached {
            return Ok(granted);
        }
        drop(cached);

        self.ensure_permission().await
    }

    async fn update_config(&self, config: domain::NotificationConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

impl Clone for NotificationService {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            config: self.config.clone(),
            permission_granted: self.permission_granted.clone(),
        }
    }
}
