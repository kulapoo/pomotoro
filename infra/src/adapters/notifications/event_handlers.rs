use async_trait::async_trait;
use domain::timer::events::{
    BreakSessionCompleted, BreakSessionStarted, Paused as TimerPaused,
    Started as TimerStarted, WorkSessionCompleted,
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
                from: phase_completed.completed_phase.clone(),
                to: phase_completed.next_phase.clone(),
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
            let task_name =
                if let Some(entity_id) = &timer_started.active_entity_id {
                    if let Some(task) = self
                        .task_repository
                        .get_all()
                        .await?
                        .into_iter()
                        .find(|t| t.id == *entity_id)
                    {
                        Some(task.name)
                    } else {
                        None
                    }
                } else {
                    None
                };

            let notification_event = NotificationEvent::SessionStarted {
                phase: timer_started.phase.clone(),
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
                phase: timer_paused.phase.clone(),
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

pub struct WorkSessionCompletedNotificationHandler {
    notification_service: Arc<dyn NotificationServiceTrait>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
}

impl WorkSessionCompletedNotificationHandler {
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
impl EventHandler for WorkSessionCompletedNotificationHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<WorkSessionCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(work_completed) =
            event.as_any().downcast_ref::<WorkSessionCompleted>()
        {
            let task_name =
                if let Some(entity_id) = &work_completed.active_entity_id {
                    if let Some(task) = self
                        .task_repository
                        .get_all()
                        .await?
                        .into_iter()
                        .find(|t| t.id.to_string() == *entity_id)
                    {
                        Some(task.name)
                    } else {
                        None
                    }
                } else {
                    None
                };

            let notification_event = NotificationEvent::WorkSessionCompleted {
                session_number: work_completed.session_count,
                task_name,
            };
            self.notification_service
                .send_notification(notification_event)
                .await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "WorkSessionCompletedNotificationHandler"
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
        TypeId::of::<BreakSessionStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(break_started) =
            event.as_any().downcast_ref::<BreakSessionStarted>()
        {
            let notification_event = NotificationEvent::BreakStarted {
                break_type: break_started.phase.clone(),
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
        TypeId::of::<BreakSessionCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(break_completed) =
            event.as_any().downcast_ref::<BreakSessionCompleted>()
        {
            let notification_event = NotificationEvent::BreakCompleted {
                break_type: break_completed.phase.clone(),
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
        WorkSessionCompletedNotificationHandler::new(
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
