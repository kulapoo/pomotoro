use std::sync::Mutex;
use async_trait::async_trait;
use domain::{
    Result, TaskId, Task, TimerStatus,
    timer::{Timer, TimerService, TimerState, Phase},
};

/// Mock timer service for testing
pub struct MockTimerService {
    timer: Mutex<Timer>,
    method_calls: Mutex<Vec<String>>,
    start_count: Mutex<usize>,
    stop_count: Mutex<usize>,
    pause_count: Mutex<usize>,
    reset_count: Mutex<usize>,
    skip_count: Mutex<usize>,
}

impl MockTimerService {
    pub fn new(initial_state: TimerState) -> Self {
        let config = initial_state.configuration().clone();
        let timer = Timer::new(config);
        // TODO: We lose the initial state details (session count, active entity, etc.)
        // but this keeps the mock simpler and avoids sync issues
        Self {
            timer: Mutex::new(timer),
            method_calls: Mutex::new(Vec::new()),
            start_count: Mutex::new(0),
            stop_count: Mutex::new(0),
            pause_count: Mutex::new(0),
            reset_count: Mutex::new(0),
            skip_count: Mutex::new(0),
        }
    }

    pub fn method_calls(&self) -> Vec<String> {
        self.method_calls.lock().unwrap().clone()
    }

    pub fn start_count(&self) -> usize {
        *self.start_count.lock().unwrap()
    }

    pub fn stop_count(&self) -> usize {
        *self.stop_count.lock().unwrap()
    }

    pub fn pause_count(&self) -> usize {
        *self.pause_count.lock().unwrap()
    }

    pub fn reset_count(&self) -> usize {
        *self.reset_count.lock().unwrap()
    }

    pub fn skip_count(&self) -> usize {
        *self.skip_count.lock().unwrap()
    }

    pub fn set_state(&self, state: TimerState) {
        // Create a new timer with the configuration from the state
        let config = state.configuration().clone();
        *self.timer.lock().unwrap() = Timer::new(config);
        // Note: We can't set the exact state, but at least the config is correct
    }

    pub fn current_state(&self) -> TimerState {
        self.timer.lock().unwrap().state().clone()
    }
}

#[async_trait]
impl TimerService for MockTimerService {
    async fn get_timer(&self) -> Result<Timer> {
        self.method_calls.lock().unwrap().push("get_timer".to_string());
        Ok(self.timer.lock().unwrap().clone())
    }

    async fn get_state(&self) -> Result<TimerState> {
        self.method_calls.lock().unwrap().push("get_state".to_string());
        Ok(self.timer.lock().unwrap().state().clone())
    }

    async fn load_state(&self) -> Result<()> {
        self.method_calls.lock().unwrap().push("load_state".to_string());
        Ok(())
    }

    async fn switch_task(
        &self,
        task_id: TaskId,
        _task: Option<&Task>,
    ) -> Result<()> {
        self.method_calls.lock().unwrap().push("switch_task".to_string());
        
        // Use the timer's set_active_entity method
        self.timer.lock().unwrap().set_active_entity(Some(task_id.to_string()))?;
        Ok(())
    }

    async fn start_timer(&self, task: Option<&Task>) -> Result<()> {
        self.method_calls.lock().unwrap().push("start_timer".to_string());
        *self.start_count.lock().unwrap() += 1;

        let mut timer = self.timer.lock().unwrap();
        
        // If timer is already running, reset it first
        if timer.is_running() {
            timer.reset()?;
        }
        
        // Set active entity - use task id if provided, otherwise use a default
        let entity_id = task.map(|t| t.id().to_string())
            .or_else(|| Some("default-task".to_string()));
        timer.set_active_entity(entity_id)?;
        
        timer.start()?;
        Ok(())
    }

    async fn stop_timer(&self) -> Result<()> {
        self.method_calls.lock().unwrap().push("stop_timer".to_string());
        *self.stop_count.lock().unwrap() += 1;

        self.timer.lock().unwrap().reset()?;
        Ok(())
    }

    async fn toggle_pause(&self) -> Result<TimerStatus> {
        self.method_calls.lock().unwrap().push("toggle_pause".to_string());
        *self.pause_count.lock().unwrap() += 1;

        let mut timer = self.timer.lock().unwrap();
        
        if timer.is_paused() {
            timer.resume()?;
            Ok(TimerStatus::Running)
        } else if timer.is_running() {
            timer.pause()?;
            Ok(TimerStatus::Paused)
        } else {
            Ok(TimerStatus::Idle)
        }
    }

    async fn reset_current_phase(&self, _task: Option<&Task>) -> Result<()> {
        self.method_calls.lock().unwrap().push("reset_current_phase".to_string());
        *self.reset_count.lock().unwrap() += 1;

        self.timer.lock().unwrap().reset()?;
        Ok(())
    }

    async fn skip_to_next_phase(
        &self,
        _task: Option<&Task>,
    ) -> Result<(Phase, Phase)> {
        self.method_calls.lock().unwrap().push("skip_to_next_phase".to_string());
        *self.skip_count.lock().unwrap() += 1;

        let mut timer = self.timer.lock().unwrap();
        let old_phase = timer.get_current_phase();
        
        timer.skip_phase()?;
        
        let new_phase = timer.get_current_phase();
        Ok((old_phase, new_phase))
    }
}

impl Default for MockTimerService {
    fn default() -> Self {
        use super::super::fixtures::TimerFixtures;
        Self::new(TimerFixtures::initial_state())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::core::fixtures::{TimerFixtures, TaskFixtures};

    #[tokio::test]
    async fn tracks_method_calls() {
        let service = MockTimerService::default();

        service.start_timer(None).await.unwrap();
        service.toggle_pause().await.unwrap();
        service.toggle_pause().await.unwrap();
        service.stop_timer().await.unwrap();

        let calls = service.method_calls();
        assert_eq!(calls, vec!["start_timer", "toggle_pause", "toggle_pause", "stop_timer"]);
    }

    #[tokio::test]
    async fn counts_operations() {
        let service = MockTimerService::default();

        service.start_timer(None).await.unwrap();
        service.start_timer(None).await.unwrap();
        service.toggle_pause().await.unwrap();
        service.reset_current_phase(None).await.unwrap();

        assert_eq!(service.start_count(), 2);
        assert_eq!(service.pause_count(), 1);
        assert_eq!(service.reset_count(), 1);
    }

    #[tokio::test]
    async fn handles_state_transitions() {
        let service = MockTimerService::default();

        // Start timer
        service.start_timer(None).await.unwrap();
        let state = service.get_state().await.unwrap();
        assert!(matches!(state, TimerState::Working { .. }));

        // Pause timer
        let status = service.toggle_pause().await.unwrap();
        assert_eq!(status, TimerStatus::Paused);

        // Resume timer
        let status = service.toggle_pause().await.unwrap();
        assert_eq!(status, TimerStatus::Running);

        // Stop timer
        service.stop_timer().await.unwrap();
        let state = service.get_state().await.unwrap();
        assert!(matches!(state, TimerState::Idle { .. }));
    }
}