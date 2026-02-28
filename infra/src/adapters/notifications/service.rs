use async_trait::async_trait;
use domain::{NotificationConfig, Phase, Result};

#[derive(Debug, Clone)]
pub struct NotificationContext {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub enum NotificationEvent {
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
    WorkPhaseCompleted {
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
            NotificationEvent::WorkPhaseCompleted { task_name } => {
                let body = if let Some(name) = task_name {
                    format!("Work session completed for \"{}\".", name)
                } else {
                    "Work session completed.".to_string()
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

/// Abstract notification service trait.
///
/// This trait allows different implementations (Tauri, libnotify, D-Bus, etc.)
/// without coupling the infrastructure layer to any specific notification system.
#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn send_notification(&self, event: NotificationEvent) -> Result<()>;
    async fn request_permission(&self) -> Result<bool>;
    async fn check_permission(&self) -> Result<bool>;
    async fn update_config(&self, config: NotificationConfig);
    fn is_enabled(&self) -> bool;
}
