use async_trait::async_trait;
use domain::timer::events::{
    BreakPhaseCompleted, BreakPhaseStarted, Paused as TimerPaused,
    Started as TimerStarted, WorkPhaseCompleted,
};
use domain::{ConfigRepository, Event, PhaseCompleted, Result, TaskCompleted};
use std::any::TypeId;
use std::sync::Arc;
use tauri::AppHandle;

use super::service::{
    NotificationEvent, NotificationService, NotificationServiceTrait,
};
use crate::adapters::events::EventHandler;
use crate::adapters::events::EventSubscriber;

pub struct PhaseCompletedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
}

impl PhaseCompletedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
    ) -> Self {
        Self {
            notification_service,
        }
    }
}

#[async_trait]
impl EventHandler for PhaseCompletedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(phase_completed) =
            event.as_any().downcast_ref::<PhaseCompleted>()
        {
            let notification_event = NotificationEvent::PhaseCompleted {
                from: phase_completed.completed_phase,
                to: phase_completed.next_phase,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PhaseCompletedNotificationHandler"
    }
}

pub struct TimerStartedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
}

impl TimerStartedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
        task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    ) -> Self {
        Self {
            notification_service,
            task_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerStartedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_started) =
            event.as_any().downcast_ref::<TimerStarted>()
        {
            let task_name = if let Some(task) = self
                .task_repository
                .get_all()
                .await?
                .into_iter()
                .find(|t| t.id == timer_started.task_id)
            {
                Some(task.name)
            } else {
                None
            };

            let notification_event = NotificationEvent::SessionStarted {
                phase: timer_started.phase,
                task_name,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerStartedNotificationHandler"
    }
}

pub struct TimerPausedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
}

impl TimerPausedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
    ) -> Self {
        Self {
            notification_service,
        }
    }
}

#[async_trait]
impl EventHandler for TimerPausedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerPaused>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_paused) = event.as_any().downcast_ref::<TimerPaused>()
        {
            let notification_event = NotificationEvent::TimerPaused {
                phase: timer_paused.phase,
                remaining_seconds: timer_paused.remaining_seconds,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerPausedNotificationHandler"
    }
}

pub struct TaskCompletedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
}

impl TaskCompletedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
    ) -> Self {
        Self {
            notification_service,
        }
    }
}

#[async_trait]
impl EventHandler for TaskCompletedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TaskCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(task_completed) =
            event.as_any().downcast_ref::<TaskCompleted>()
        {
            let notification_event = NotificationEvent::TaskCompleted {
                task_name: task_completed.task_id.to_string(),
                total_sessions: task_completed.total_sessions as u32,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TaskCompletedNotificationHandler"
    }
}

pub struct WorkPhaseCompletedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
}

impl WorkPhaseCompletedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
        task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    ) -> Self {
        Self {
            notification_service,
            task_repository,
        }
    }
}

#[async_trait]
impl EventHandler for WorkPhaseCompletedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<WorkPhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(work_completed) =
            event.as_any().downcast_ref::<WorkPhaseCompleted>()
        {
            let task_name = if let Some(task) = self
                .task_repository
                .get_all()
                .await?
                .into_iter()
                .find(|t| t.id == work_completed.task_id)
            {
                Some(task.name)
            } else {
                None
            };

            let notification_event = NotificationEvent::WorkPhaseCompleted {
                task_name,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "WorkPhaseCompletedNotificationHandler"
    }
}

pub struct BreakStartedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
}

impl BreakStartedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
    ) -> Self {
        Self {
            notification_service,
        }
    }
}

#[async_trait]
impl EventHandler for BreakStartedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<BreakPhaseStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(break_started) =
            event.as_any().downcast_ref::<BreakPhaseStarted>()
        {
            let notification_event = NotificationEvent::BreakStarted {
                break_type: break_started.phase,
                duration_seconds: break_started.duration_seconds,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "BreakStartedNotificationHandler"
    }
}

pub struct BreakCompletedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
}

impl BreakCompletedNotificationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationServiceTrait>,
    ) -> Self {
        Self {
            notification_service,
        }
    }
}

#[async_trait]
impl EventHandler for BreakCompletedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<BreakPhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(break_completed) =
            event.as_any().downcast_ref::<BreakPhaseCompleted>()
        {
            let notification_event = NotificationEvent::BreakCompleted {
                break_type: break_completed.phase,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "BreakCompletedNotificationHandler"
    }
}

pub async fn register_notification_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: AppHandle,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
) -> Result<()> {
    let config = config_repository.get_config().await.map_err(|e| {
        eprintln!(
            "Failed to get config in register_notification_handlers: {:?}",
            e
        );
        e
    })?;

    let notification_service: Arc<dyn NotificationServiceTrait> =
        Arc::new(NotificationService::new(app_handle, config.notification));

    let _ = event_bus.subscribe(Box::new(
        PhaseCompletedNotificationHandler::new(notification_service.clone()),
    ));

    let _ =
        event_bus.subscribe(Box::new(TimerStartedNotificationHandler::new(
            notification_service.clone(),
            task_repository.clone(),
        )));

    let _ = event_bus.subscribe(Box::new(TimerPausedNotificationHandler::new(
        notification_service.clone(),
    )));

    let _ = event_bus.subscribe(Box::new(
        TaskCompletedNotificationHandler::new(notification_service.clone()),
    ));

    let _ = event_bus.subscribe(Box::new(
        WorkPhaseCompletedNotificationHandler::new(
            notification_service.clone(),
            task_repository.clone(),
        ),
    ));

    let _ = event_bus.subscribe(Box::new(
        BreakStartedNotificationHandler::new(notification_service.clone()),
    ));

    let _ = event_bus.subscribe(Box::new(
        BreakCompletedNotificationHandler::new(notification_service.clone()),
    ));

    Ok(())
}
