use domain::{Phase, Task, timer::TimerService as TimerServiceTrait};
use infra::adapters::InMemoryTimerService;
use std::sync::Arc;
use std::time::Duration;

pub struct TimerTestService {
    service: Arc<InMemoryTimerService>,
}

impl TimerTestService {
    pub fn new() -> Self {
        Self {
            service: Arc::new(InMemoryTimerService::new()),
        }
    }

    pub async fn setup_with_task(&self, task: &Task) {
        let _ =
            TimerServiceTrait::switch_task(&*self.service, task.id, Some(task))
                .await;
    }

    pub async fn start_work_session(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check current state and act accordingly
        let state = TimerServiceTrait::get_state(&*self.service).await?;
        if state.is_paused() {
            // Resume from paused state
            TimerServiceTrait::toggle_pause(&*self.service).await?;
        } else if !state.is_running() {
            // Start from stopped state
            TimerServiceTrait::start_timer(&*self.service, None).await?;
        }
        Ok(())
    }

    pub async fn pause_timer(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.service.toggle_pause().await?;
        Ok(())
    }

    pub async fn stop_timer(&self) -> Result<(), Box<dyn std::error::Error>> {
        TimerServiceTrait::stop_timer(&*self.service).await?;
        Ok(())
    }

    pub async fn reset_current_phase(
        &self,
        task: Option<&Task>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ =
            TimerServiceTrait::reset_current_phase(&*self.service, task).await;
        Ok(())
    }

    pub async fn skip_to_next_phase(
        &self,
        task: Option<&Task>,
    ) -> Result<(Phase, Phase), Box<dyn std::error::Error>> {
        let result =
            TimerServiceTrait::skip_to_next_phase(&*self.service, task).await;
        Ok(result?)
    }

    pub async fn wait_for_seconds(&self, seconds: u64) {
        tokio::time::sleep(Duration::from_millis(seconds * 100)).await;
    }

    pub async fn force_complete_session(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Force complete the current session by transitioning to next phase
        TimerServiceTrait::skip_to_next_phase(&*self.service, None).await?;
        Ok(())
    }
}

impl std::ops::Deref for TimerTestService {
    type Target = InMemoryTimerService;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}
