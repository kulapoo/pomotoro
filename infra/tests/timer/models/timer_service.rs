use domain::{Phase, Task, TimerStatus};
use infra::adapters::TimerService;
use std::sync::Arc;
use std::time::Duration;

pub struct TimerTestService {
    service: Arc<TimerService>,
}

impl TimerTestService {
    pub fn new() -> Self {
        Self {
            service: Arc::new(TimerService::new()),
        }
    }

    pub async fn setup_with_task(&self, task: &Task) {
        self.service.switch_task(task.id, Some(task)).await;
    }

    pub async fn start_work_session(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.service.set_status(TimerStatus::Running).await;
        Ok(())
    }

    pub async fn pause_timer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.service.set_status(TimerStatus::Paused).await;
        Ok(())
    }

    pub async fn stop_timer(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.service.stop_timer().await;
        let _ = self.service.set_status(TimerStatus::Stopped).await;
        Ok(())
    }

    pub async fn reset_current_phase(
        &self,
        task: Option<&Task>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.service.reset_current_phase(task).await;
        Ok(())
    }

    pub async fn skip_to_next_phase(
        &self,
        task: Option<&Task>,
    ) -> Result<(Phase, Phase), Box<dyn std::error::Error>> {
        let result = self.service.skip_to_next_phase(task).await;
        Ok(result?)
    }

    pub async fn wait_for_seconds(&self, seconds: u64) {
        tokio::time::sleep(Duration::from_millis(seconds * 100)).await;
    }

    pub async fn force_complete_session(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Force complete the current session by transitioning to next phase
        self.service.skip_to_next_phase(None).await?;
        Ok(())
    }

}

impl std::ops::Deref for TimerTestService {
    type Target = TimerService;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}
