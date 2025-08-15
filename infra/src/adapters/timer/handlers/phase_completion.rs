use async_trait::async_trait;
use domain::{
    events::DomainEventHandler,
    timer::events::PhaseCompleted,
    DomainEvent, Phase, Result,
};
use std::any::TypeId;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PhaseCompletionHandler {
    audio_player: Arc<RwLock<dyn AudioNotificationPlayer>>,
    notification_service: Arc<dyn NotificationService>,
}

impl PhaseCompletionHandler {
    pub fn new(
        audio_player: Arc<RwLock<dyn AudioNotificationPlayer>>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            audio_player,
            notification_service,
        }
    }

    async fn handle_phase_transition(&self, event: &PhaseCompleted) -> Result<()> {
        match event.completed_phase {
            Phase::Work => {
                self.audio_player
                    .write()
                    .await
                    .play_work_complete()
                    .await?;

                self.notification_service
                    .notify(&format!(
                        "Work session {} completed! Time for a {} break.",
                        event.session_count,
                        if event.next_phase == Phase::LongBreak {
                            "long"
                        } else {
                            "short"
                        }
                    ))
                    .await?;
            }
            Phase::ShortBreak => {
                self.audio_player
                    .write()
                    .await
                    .play_break_complete()
                    .await?;

                self.notification_service
                    .notify("Break finished! Ready to focus again?")
                    .await?;
            }
            Phase::LongBreak => {
                self.audio_player
                    .write()
                    .await
                    .play_break_complete()
                    .await?;

                self.notification_service
                    .notify("Long break complete! Feeling refreshed?")
                    .await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl DomainEventHandler for PhaseCompletionHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn DomainEvent>) -> Result<()> {
        if let Some(phase_completed) = event
            .as_any()
            .downcast_ref::<PhaseCompleted>()
        {
            self.handle_phase_transition(phase_completed).await?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PhaseCompletionHandler"
    }
}

#[async_trait]
pub trait AudioNotificationPlayer: Send + Sync {
    async fn play_work_complete(&mut self) -> Result<()>;
    async fn play_break_complete(&mut self) -> Result<()>;
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn notify(&self, message: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{TaskId, Phase};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockAudioPlayer {
        work_complete_calls: Arc<AtomicUsize>,
        break_complete_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl AudioNotificationPlayer for MockAudioPlayer {
        async fn play_work_complete(&mut self) -> Result<()> {
            self.work_complete_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn play_break_complete(&mut self) -> Result<()> {
            self.break_complete_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct MockNotificationService {
        notifications: Arc<RwLock<Vec<String>>>,
    }

    #[async_trait]
    impl NotificationService for MockNotificationService {
        async fn notify(&self, message: &str) -> Result<()> {
            self.notifications.write().await.push(message.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_handle_work_phase_completion() {
        let work_calls = Arc::new(AtomicUsize::new(0));
        let break_calls = Arc::new(AtomicUsize::new(0));

        let audio_player = Arc::new(RwLock::new(MockAudioPlayer {
            work_complete_calls: work_calls.clone(),
            break_complete_calls: break_calls.clone(),
        }));

        let notifications = Arc::new(RwLock::new(Vec::new()));
        let notification_service = Arc::new(MockNotificationService {
            notifications: notifications.clone(),
        });

        let handler = PhaseCompletionHandler::new(audio_player, notification_service);

        let event = PhaseCompleted::new(
            Some(TaskId::new()),
            Phase::Work,
            Phase::ShortBreak,
            1,
            1,
            1,
        );

        handler.handle(Box::new(event)).await.unwrap();

        assert_eq!(work_calls.load(Ordering::SeqCst), 1);
        assert_eq!(break_calls.load(Ordering::SeqCst), 0);

        let notifs = notifications.read().await;
        assert_eq!(notifs.len(), 1);
        assert!(notifs[0].contains("Work session 1 completed"));
    }

    #[tokio::test]
    async fn should_handle_break_phase_completion() {
        let work_calls = Arc::new(AtomicUsize::new(0));
        let break_calls = Arc::new(AtomicUsize::new(0));

        let audio_player = Arc::new(RwLock::new(MockAudioPlayer {
            work_complete_calls: work_calls.clone(),
            break_complete_calls: break_calls.clone(),
        }));

        let notifications = Arc::new(RwLock::new(Vec::new()));
        let notification_service = Arc::new(MockNotificationService {
            notifications: notifications.clone(),
        });

        let handler = PhaseCompletionHandler::new(audio_player, notification_service);

        let event = PhaseCompleted::new(
            Some(TaskId::new()),
            Phase::ShortBreak,
            Phase::Work,
            2,
            2,
            1,
        );

        handler.handle(Box::new(event)).await.unwrap();

        assert_eq!(work_calls.load(Ordering::SeqCst), 0);
        assert_eq!(break_calls.load(Ordering::SeqCst), 1);

        let notifs = notifications.read().await;
        assert_eq!(notifs.len(), 1);
        assert!(notifs[0].contains("Break finished"));
    }
}