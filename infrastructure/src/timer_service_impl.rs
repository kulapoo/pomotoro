use std::sync::{Arc, Mutex};
use domain::{
    Result, TimerState, TimerConfiguration, TaskId, Task, TimerStatus, Phase,
    timer::Timer,
    EventPublisher,
};

/// Thread-safe implementation of timer service
/// Handles infrastructure concerns like concurrency
pub struct ThreadSafeTimerService {
    timer: Arc<Mutex<Timer>>,
}

impl ThreadSafeTimerService {
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self {
            timer: Arc::new(Mutex::new(Timer::new(configuration))),
        }
    }
    
    pub fn with_event_publisher(self, publisher: Box<dyn EventPublisher>) -> Self {
        let config = {
            let timer = self.timer.lock().unwrap();
            timer.state().configuration().clone()
        };
        let new_timer = Timer::new(config).with_event_publisher(publisher);
        *self.timer.lock().unwrap() = new_timer;
        self
    }
    
    pub async fn get_state(&self) -> Result<TimerState> {
        Ok(self.timer.lock().unwrap().state().clone())
    }
    
    
    pub async fn start(&self) -> Result<()> {
        Ok(self.timer.lock().unwrap().start()?)
    }
    
    pub async fn pause(&self) -> Result<()> {
        Ok(self.timer.lock().unwrap().pause()?)
    }
    
    pub async fn resume(&self) -> Result<()> {
        Ok(self.timer.lock().unwrap().resume()?)
    }
    
    pub async fn reset(&self) -> Result<()> {
        Ok(self.timer.lock().unwrap().reset()?)
    }
    
    pub async fn skip_phase(&self) -> Result<()> {
        Ok(self.timer.lock().unwrap().skip_phase()?)
    }
    
    pub async fn tick(&self) -> Result<bool> {
        Ok(self.timer.lock().unwrap().tick()?)
    }
    
    pub async fn set_active_task(&self, task_id: Option<TaskId>) -> Result<()> {
        let task_id_str = task_id.map(|id| id.to_string());
        Ok(self.timer.lock().unwrap().set_active_entity(task_id_str)?)
    }
    
    pub async fn update_configuration(&self, configuration: TimerConfiguration) -> Result<()> {
        Ok(self.timer.lock().unwrap().update_configuration(configuration)?)
    }
    
    pub async fn toggle_pause(&self) -> Result<TimerStatus> {
        let mut timer = self.timer.lock().unwrap();
        if timer.is_running() {
            timer.pause()?;
            Ok(TimerStatus::Paused)
        } else if timer.is_paused() {
            timer.resume()?;
            Ok(TimerStatus::Running)
        } else {
            Ok(timer.state().status())
        }
    }
    
    pub async fn switch_task(&self, task_id: TaskId, _task: Option<&Task>) -> Result<()> {
        self.set_active_task(Some(task_id)).await
    }
    
    pub async fn start_timer(&self, _task: Option<&Task>) -> Result<()> {
        self.start().await
    }
    
    pub async fn stop_timer(&self) -> Result<()> {
        self.reset().await
    }
    
    pub async fn reset_current_phase(&self, _task: Option<&Task>) -> Result<()> {
        self.reset().await
    }
    
    pub async fn skip_to_next_phase(&self, _task: Option<&Task>) -> Result<(Phase, Phase)> {
        let old_phase = self.timer.lock().unwrap().state().phase();
        self.skip_phase().await?;
        let new_phase = self.timer.lock().unwrap().state().phase();
        Ok((old_phase, new_phase))
    }
    
    pub async fn load_state(&self) -> Result<()> {
        // This would load from persistence in a real implementation
        Ok(())
    }
}

/// Domain-compliant timer service without infrastructure concerns
pub struct DomainTimerService {
    timer: Timer,
}

impl DomainTimerService {
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self {
            timer: Timer::new(configuration),
        }
    }
    
    pub fn with_event_publisher(mut self, publisher: Box<dyn EventPublisher>) -> Self {
        self.timer = self.timer.with_event_publisher(publisher);
        self
    }
    
    pub fn get_state(&self) -> &TimerState {
        self.timer.state()
    }
    
    pub fn start(&mut self) -> Result<()> {
        Ok(self.timer.start()?)
    }
    
    pub fn pause(&mut self) -> Result<()> {
        Ok(self.timer.pause()?)
    }
    
    pub fn resume(&mut self) -> Result<()> {
        Ok(self.timer.resume()?)
    }
    
    pub fn reset(&mut self) -> Result<()> {
        Ok(self.timer.reset()?)
    }
    
    pub fn skip_phase(&mut self) -> Result<()> {
        Ok(self.timer.skip_phase()?)
    }
    
    pub fn tick(&mut self) -> Result<bool> {
        Ok(self.timer.tick()?)
    }
    
    pub fn set_active_task(&mut self, task_id: Option<TaskId>) -> Result<()> {
        let task_id_str = task_id.map(|id| id.to_string());
        Ok(self.timer.set_active_entity(task_id_str)?)
    }
    
    pub fn update_configuration(&mut self, configuration: TimerConfiguration) -> Result<()> {
        Ok(self.timer.update_configuration(configuration)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn should_handle_concurrent_access() {
        let service = ThreadSafeTimerService::new(TimerConfiguration::default());
        
        // Clone for concurrent access
        let service1 = Arc::new(service);
        let service2 = service1.clone();
        
        // Set task and start from one thread
        let task_id = TaskId::new();
        service1.set_active_task(Some(task_id)).await.unwrap();
        service1.start().await.unwrap();
        
        // Access state from another thread
        let state = service2.get_state().await.unwrap();
        assert!(state.is_running());
    }
    
    #[test]
    fn should_work_without_concurrency() {
        let mut service = DomainTimerService::new(TimerConfiguration::default());
        
        let task_id = TaskId::new();
        service.set_active_task(Some(task_id)).unwrap();
        service.start().unwrap();
        
        assert!(service.get_state().is_running());
    }
}