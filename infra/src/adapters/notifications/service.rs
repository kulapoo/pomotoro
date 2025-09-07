use async_trait::async_trait;
use domain::{NotificationConfig, Phase, Result};
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_notification::{NotificationExt, PermissionState};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct NotificationContext {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub enum NotificationEvent {
    PhaseCompleted {
        from: Phase,
        to: Phase,
    },
    SessionStarted {
        phase: Phase,
        task_name: Option<String>,
    },
    TaskCompleted {
        task_name: String,
        total_sessions: u32,
    },
    TimerPaused {
        phase: Phase,
        remaining_seconds: u32,
    },
    TimerResumed {
        phase: Phase,
        remaining_seconds: u32,
    },
    WorkSessionCompleted {
        session_number: u32,
        task_name: Option<String>,
    },
    BreakStarted {
        break_type: Phase,
        duration_seconds: u32,
    },
    BreakCompleted {
        break_type: Phase,
    },
}

impl NotificationEvent {
    pub fn to_context(&self) -> NotificationContext {
        match self {
            NotificationEvent::PhaseCompleted { from, to } => {
                let (title, body) = match (from, to) {
                    (Phase::Work, Phase::ShortBreak) => (
                        "Great work!".to_string(),
                        "Time for a short break. Rest your eyes and stretch!".to_string(),
                    ),
                    (Phase::Work, Phase::LongBreak) => (
                        "Excellent!".to_string(),
                        "You've completed your focus sessions! Time for a long break.".to_string(),
                    ),
                    (Phase::ShortBreak, Phase::Work) | (Phase::LongBreak, Phase::Work) => (
                        "Break's over!".to_string(),
                        "Ready to focus? Let's get back to work!".to_string(),
                    ),
                    _ => return NotificationContext {
                        title: "Phase Changed".to_string(),
                        body: format!("Switched from {:?} to {:?}", from, to),
                        icon: None,
                    },
                };
                NotificationContext {
                    title,
                    body,
                    icon: None,
                }
            }
            NotificationEvent::SessionStarted { phase, task_name } => {
                let title = match phase {
                    Phase::Work => "Focus Session Started".to_string(),
                    Phase::ShortBreak => "Short Break Started".to_string(),
                    Phase::LongBreak => "Long Break Started".to_string(),
                };
                let body = if let Some(name) = task_name {
                    format!("Working on: {}", name)
                } else {
                    format!("{:?} session has started", phase)
                };
                NotificationContext {
                    title,
                    body,
                    icon: None,
                }
            }
            NotificationEvent::TaskCompleted {
                task_name,
                total_sessions,
            } => NotificationContext {
                title: "Task Completed!".to_string(),
                body: format!(
                    "\"{}\" completed with {} sessions",
                    task_name, total_sessions
                ),
                icon: None,
            },
            NotificationEvent::TimerPaused {
                phase,
                remaining_seconds,
            } => NotificationContext {
                title: "Timer Paused".to_string(),
                body: format!(
                    "{:?} paused with {} minutes remaining",
                    phase,
                    remaining_seconds / 60
                ),
                icon: None,
            },
            NotificationEvent::TimerResumed {
                phase,
                remaining_seconds,
            } => NotificationContext {
                title: "Timer Resumed".to_string(),
                body: format!(
                    "{:?} resumed with {} minutes remaining",
                    phase,
                    remaining_seconds / 60
                ),
                icon: None,
            },
            NotificationEvent::WorkSessionCompleted {
                session_number,
                task_name,
            } => {
                let body = if let Some(name) = task_name {
                    format!(
                        "Session {} completed for \"{}\"",
                        session_number, name
                    )
                } else {
                    format!("Work session {} completed", session_number)
                };
                NotificationContext {
                    title: "Session Complete!".to_string(),
                    body,
                    icon: None,
                }
            }
            NotificationEvent::BreakStarted {
                break_type,
                duration_seconds,
            } => {
                let duration_minutes = duration_seconds / 60;
                let (title, body) = match break_type {
                    Phase::ShortBreak => (
                        "Short Break".to_string(),
                        format!(
                            "Take {} minutes to rest and recharge",
                            duration_minutes
                        ),
                    ),
                    Phase::LongBreak => (
                        "Long Break".to_string(),
                        format!(
                            "Take {} minutes to relax and refresh",
                            duration_minutes
                        ),
                    ),
                    _ => {
                        return NotificationContext {
                            title: "Break Started".to_string(),
                            body: format!(
                                "Time for a {}-minute break",
                                duration_minutes
                            ),
                            icon: None,
                        };
                    }
                };
                NotificationContext {
                    title,
                    body,
                    icon: None,
                }
            }
            NotificationEvent::BreakCompleted { break_type } => {
                let body = match break_type {
                    Phase::ShortBreak => {
                        "Short break is over. Ready to focus?".to_string()
                    }
                    Phase::LongBreak => {
                        "Long break is over. Let's get back to work!"
                            .to_string()
                    }
                    _ => "Break is over".to_string(),
                };
                NotificationContext {
                    title: "Break Complete".to_string(),
                    body,
                    icon: None,
                }
            }
        }
    }
}

#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn send_notification(&self, event: NotificationEvent) -> Result<()>;
    async fn request_permission(&self) -> Result<bool>;
    async fn check_permission(&self) -> Result<bool>;
    async fn update_config(&self, config: NotificationConfig);
    fn is_enabled(&self) -> bool;
}

pub struct NotificationService {
    app_handle: AppHandle,
    config: Arc<RwLock<NotificationConfig>>,
    permission_granted: Arc<RwLock<Option<bool>>>,
}

impl NotificationService {
    pub fn new(app_handle: AppHandle, config: NotificationConfig) -> Self {
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
            NotificationEvent::PhaseCompleted { .. }
            | NotificationEvent::WorkSessionCompleted { .. }
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

    async fn ensure_permission(&self) -> Result<bool> {
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
                    match self.app_handle.notification().request_permission() {
                        Ok(PermissionState::Granted) => true,
                        _ => false,
                    }
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
    async fn send_notification(&self, event: NotificationEvent) -> Result<()> {
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
                        tracing::warn!("Failed to show notification: {}", e);
                    }
                }
            }
        }

        // Sound notifications will be handled separately through the audio service
        // when the notification events are published

        Ok(())
    }

    async fn request_permission(&self) -> Result<bool> {
        self.ensure_permission().await
    }

    async fn check_permission(&self) -> Result<bool> {
        let cached = self.permission_granted.read().await;
        if let Some(granted) = *cached {
            return Ok(granted);
        }
        drop(cached);

        self.ensure_permission().await
    }

    async fn update_config(&self, config: NotificationConfig) {
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
