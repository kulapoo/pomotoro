use std::sync::Mutex;
use async_trait::async_trait;
use domain::{
    Result, TaskId, Task, TimerStatus,
    timer::{TimerService, TimerState, Phase},
};

/// Mock timer service for testing
pub struct MockTimerService {
    state: Mutex<TimerState>,
    method_calls: Mutex<Vec<String>>,
    start_count: Mutex<usize>,
    pause_count: Mutex<usize>,
    reset_count: Mutex<usize>,
    skip_count: Mutex<usize>,
}

impl MockTimerService {
    pub fn new(initial_state: TimerState) -> Self {
        Self {
            state: Mutex::new(initial_state),
            method_calls: Mutex::new(Vec::new()),
            start_count: Mutex::new(0),
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
        *self.state.lock().unwrap() = state;
    }

    pub fn is_running(&self) -> bool {
        self.state.lock().unwrap().is_running
    }

    pub fn current_phase(&self) -> Phase {
        self.state.lock().unwrap().phase.clone()
    }

    pub fn clear_calls(&self) {
        self.method_calls.lock().unwrap().clear();
        *self.start_count.lock().unwrap() = 0;
        *self.pause_count.lock().unwrap() = 0;
        *self.reset_count.lock().unwrap() = 0;
        *self.skip_count.lock().unwrap() = 0;
    }
}

#[async_trait]
impl TimerService for MockTimerService {
    async fn get_state(&self) -> Result<TimerState> {
        self.method_calls.lock().unwrap().push("get_state".to_string());
        Ok(self.state.lock().unwrap().clone())
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
        self.method_calls.lock().unwrap().push(format!("switch_task:{}", task_id));
        self.state.lock().unwrap().task_id = Some(task_id);
        Ok(())
    }

    async fn start_timer(&self, task: Option<&Task>) -> Result<()> {
        self.method_calls.lock().unwrap().push(
            format!("start_timer:{}", task.map(|t| t.id().to_string()).unwrap_or_else(|| "none".to_string()))
        );
        *self.start_count.lock().unwrap() += 1;
        
        let mut state = self.state.lock().unwrap();
        state.is_running = true;
        state.task_id = task.map(|t| t.id());
        Ok(())
    }

    async fn stop_timer(&self) -> Result<()> {
        self.method_calls.lock().unwrap().push("stop_timer".to_string());
        *self.pause_count.lock().unwrap() += 1;
        
        self.state.lock().unwrap().is_running = false;
        Ok(())
    }

    async fn toggle_pause(&self) -> Result<TimerStatus> {
        self.method_calls.lock().unwrap().push("toggle_pause".to_string());
        
        let mut state = self.state.lock().unwrap();
        state.is_running = !state.is_running;
        if state.is_running {
            Ok(TimerStatus::Running)
        } else {
            Ok(TimerStatus::Paused)
        }
    }

    async fn reset_current_phase(&self, _task: Option<&Task>) -> Result<()> {
        self.method_calls.lock().unwrap().push("reset_current_phase".to_string());
        *self.reset_count.lock().unwrap() += 1;
        
        let mut state = self.state.lock().unwrap();
        state.is_running = false;
        state.phase = Phase::Work;
        state.remaining = state.config.work_duration;
        state.session_count = 0;
        state.task_id = None;
        Ok(())
    }

    async fn skip_to_next_phase(
        &self,
        _task: Option<&Task>,
    ) -> Result<(Phase, Phase)> {
        self.method_calls.lock().unwrap().push("skip_to_next_phase".to_string());
        *self.skip_count.lock().unwrap() += 1;
        
        let mut state = self.state.lock().unwrap();
        let old_phase = state.phase.clone();
        
        // Simple phase transition for testing
        state.phase = match state.phase {
            Phase::Work => {
                state.session_count += 1;
                if state.session_count % state.config.sessions_until_long_break == 0 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };
        
        // Reset remaining time for new phase
        state.remaining = match state.phase {
            Phase::Work => state.config.work_duration,
            Phase::ShortBreak => state.config.short_break_duration,
            Phase::LongBreak => state.config.long_break_duration,
        };
        
        Ok((old_phase, state.phase.clone()))
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
    use crate::core::fixtures::{TimerFixtures, TaskFixtures};

    #[tokio::test]
    async fn tracks_method_calls() {
        let service = MockTimerService::default();
        
        service.start_timer(None).await.unwrap();
        service.stop_timer().await.unwrap();
        service.skip_to_next_phase(None).await.unwrap();
        service.reset_current_phase(None).await.unwrap();
        
        let calls = service.method_calls();
        assert_eq!(calls[0], "start_timer:none");
        assert_eq!(calls[1], "stop_timer");
        assert_eq!(calls[2], "skip_to_next_phase");
        assert_eq!(calls[3], "reset_current_phase");
    }

    #[tokio::test]
    async fn counts_operations() {
        let service = MockTimerService::default();
        
        service.start_timer(None).await.unwrap();
        service.start_timer(None).await.unwrap();
        service.stop_timer().await.unwrap();
        
        assert_eq!(service.start_count(), 2);
        assert_eq!(service.pause_count(), 1);
    }

    #[tokio::test]
    async fn manages_state() {
        let service = MockTimerService::default();
        
        assert!(!service.is_running());
        
        service.start_timer(None).await.unwrap();
        assert!(service.is_running());
        
        service.stop_timer().await.unwrap();
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn handles_phase_transitions() {
        let service = MockTimerService::default();
        
        assert_eq!(service.current_phase(), Phase::Work);
        
        service.skip_to_next_phase(None).await.unwrap();
        assert_eq!(service.current_phase(), Phase::ShortBreak);
        
        service.skip_to_next_phase(None).await.unwrap();
        assert_eq!(service.current_phase(), Phase::Work);
    }
}