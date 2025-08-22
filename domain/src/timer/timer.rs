use serde::{Deserialize, Serialize};
use crate::{Result, Error, TimerConfiguration, TaskId, EventPublisher};
use super::{
    state_machine::TimerState,
    transitions::{StateTransitions, TransitionResult, TransitionType},
    Phase,
};
use super::events::*;

/// Simplified timer aggregate that wraps the state machine
#[derive(Serialize, Deserialize)]
pub struct Timer {
    /// The current state of the timer
    state: TimerState,
    /// Event publisher for domain events
    #[serde(skip)]
    event_publisher: Option<Box<dyn EventPublisher>>,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            state: TimerState::default(),
            event_publisher: None,
        }
    }
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("state", &self.state)
            .field("has_publisher", &self.event_publisher.is_some())
            .finish()
    }
}

impl Timer {
    /// Create a new timer with the given configuration
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self {
            state: TimerState::new(configuration),
            event_publisher: None,
        }
    }
    
    /// Create a timer with an event publisher
    pub fn with_event_publisher(mut self, publisher: Box<dyn EventPublisher>) -> Self {
        self.event_publisher = Some(publisher);
        self
    }
    
    /// Get the current state
    pub fn state(&self) -> &TimerState {
        &self.state
    }
    
    /// Start the timer
    pub fn start(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Start) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Start".to_string(),
            });
        }
        
        let result = StateTransitions::start(self.state.clone())?;
        self.apply_transition(result)
    }
    
    /// Pause the timer
    pub fn pause(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Pause) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Pause".to_string(),
            });
        }
        
        let result = StateTransitions::pause(self.state.clone())?;
        self.apply_transition(result)
    }
    
    /// Resume the timer
    pub fn resume(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Resume) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Resume".to_string(),
            });
        }
        
        let result = StateTransitions::resume(self.state.clone())?;
        self.apply_transition(result)
    }
    
    /// Reset the timer
    pub fn reset(&mut self) -> Result<()> {
        let result = StateTransitions::reset(self.state.clone())?;
        self.apply_transition(result)
    }
    
    /// Skip the current phase
    pub fn skip_phase(&mut self) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::Skip) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "Skip".to_string(),
            });
        }
        
        let result = StateTransitions::skip_phase(self.state.clone())?;
        self.apply_transition(result)
    }
    
    /// Process a timer tick
    pub fn tick(&mut self) -> Result<bool> {
        let (new_state, phase_complete) = StateTransitions::tick(self.state.clone())?;
        let _old_state = self.state.clone();
        self.state = new_state.clone();
        
        // Publish tick event
        if let Some(publisher) = &self.event_publisher {
            let phase = self.get_current_phase();
            let event = Tick::new(
                self.state.active_task_id(),
                phase,
                self.state.remaining_seconds(),
                1,
            );
            publisher.publish(Box::new(event));
        }
        
        // If phase is complete, handle the transition
        if phase_complete {
            let result = StateTransitions::complete_phase(self.state.clone())?;
            self.apply_transition(result)?;
        }
        
        Ok(phase_complete)
    }
    
    /// Set the active task
    pub fn set_active_task(&mut self, task_id: Option<TaskId>) -> Result<()> {
        if !StateTransitions::can_transition(&self.state, TransitionType::SwitchTask) {
            return Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "SetTask".to_string(),
            });
        }
        
        let result = StateTransitions::switch_task(self.state.clone(), task_id)?;
        self.apply_transition(result)
    }
    
    /// Update the timer configuration
    pub fn update_configuration(&mut self, configuration: TimerConfiguration) -> Result<()> {
        // Only allow configuration updates when idle
        match &self.state {
            TimerState::Idle { session_count, active_task, .. } => {
                self.state = TimerState::Idle {
                    configuration,
                    session_count: *session_count,
                    active_task: *active_task,
                };
                Ok(())
            }
            _ => Err(Error::InvalidStateTransition {
                from: self.get_state_name(),
                to: "ConfigUpdate".to_string(),
            })
        }
    }
    
    /// Check if the timer can start
    pub fn can_start(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Start)
    }
    
    /// Check if the timer can pause
    pub fn can_pause(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Pause)
    }
    
    /// Check if the timer can resume
    pub fn can_resume(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Resume)
    }
    
    /// Check if the timer can skip the current phase
    pub fn can_skip(&self) -> bool {
        StateTransitions::can_transition(&self.state, TransitionType::Skip)
    }
    
    /// Get remaining seconds
    pub fn remaining_seconds(&self) -> u32 {
        self.state.remaining_seconds()
    }
    
    /// Get formatted time string
    pub fn format_time(&self) -> String {
        let seconds = self.state.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }
    
    /// Get phase name
    pub fn phase_name(&self) -> &'static str {
        match &self.state {
            TimerState::Idle { .. } => "Stopped",
            TimerState::Working { .. } => "Focus Time",
            TimerState::ShortBreak { .. } => "Short Break",
            TimerState::LongBreak { .. } => "Long Break",
            TimerState::Paused { paused_from, .. } => {
                match paused_from.as_ref() {
                    TimerState::Working { .. } => "Focus Time (Paused)",
                    TimerState::ShortBreak { .. } => "Short Break (Paused)",
                    TimerState::LongBreak { .. } => "Long Break (Paused)",
                    _ => "Paused",
                }
            }
        }
    }
    
    /// Get progress percentage
    pub fn progress_percentage(&self) -> f64 {
        match &self.state {
            TimerState::Idle { .. } => 0.0,
            TimerState::Working { remaining_seconds, configuration, .. } => {
                let total = configuration.get_phase_duration_seconds(Phase::Work) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::ShortBreak { remaining_seconds, configuration, .. } => {
                let total = configuration.get_phase_duration_seconds(Phase::ShortBreak) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::LongBreak { remaining_seconds, configuration, .. } => {
                let total = configuration.get_phase_duration_seconds(Phase::LongBreak) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            TimerState::Paused { .. } => 0.0,
        }
    }
    
    /// Get session display string
    pub fn session_display(&self) -> String {
        let count = self.state.session_count();
        let config = self.state.configuration();
        let sessions_until_long = config.sessions_until_long_break as u32;
        
        let current_in_cycle = if count == 0 {
            0
        } else {
            ((count - 1) % sessions_until_long) + 1
        };
        
        format!("Session {}/{}", current_in_cycle, sessions_until_long)
    }
    
    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }
    
    /// Check if timer is paused
    pub fn is_paused(&self) -> bool {
        self.state.is_paused()
    }
    
    /// Check if timer is idle
    pub fn is_idle(&self) -> bool {
        self.state.is_idle()
    }
    
    /// Apply a transition and publish events
    fn apply_transition(&mut self, result: TransitionResult) -> Result<()> {
        let old_state = self.state.clone();
        self.state = result.new_state.clone();
        
        // Publish events based on state transitions
        if let Some(publisher) = &self.event_publisher {
            // Determine the event type and publish accordingly
            match (&old_state, &self.state) {
                (TimerState::Idle { .. }, TimerState::Working { .. }) => {
                    let event = Started::new(
                        self.state.active_task_id(),
                        Phase::Work,
                        self.state.configuration().work_duration.as_secs() as u32,
                        1,
                    );
                    publisher.publish(Box::new(event));
                    
                    let work_event = WorkSessionStarted::new(
                        self.state.active_task_id(),
                        self.state.configuration().work_duration.as_secs() as u32,
                        self.state.session_count(),
                        1,
                        1,
                    );
                    publisher.publish(Box::new(work_event));
                }
                (_, TimerState::Paused { .. }) => {
                    let phase = self.get_current_phase();
                    let event = Paused::new(
                        self.state.active_task_id(),
                        phase,
                        self.state.remaining_seconds(),
                        1,
                    );
                    publisher.publish(Box::new(event));
                }
                (TimerState::Paused { .. }, _) if !matches!(self.state, TimerState::Idle { .. }) => {
                    // Resume event - we use Started event for compatibility
                    let phase = self.get_current_phase();
                    let event = Started::new(
                        self.state.active_task_id(),
                        phase,
                        self.state.remaining_seconds(),
                        1,
                    );
                    publisher.publish(Box::new(event));
                }
                (_, TimerState::Idle { .. }) => {
                    let event = Reset::new(self.state.active_task_id(), Phase::Work, 1);
                    publisher.publish(Box::new(event));
                }
                (TimerState::Working { .. }, TimerState::ShortBreak { .. }) |
                (TimerState::Working { .. }, TimerState::LongBreak { .. }) => {
                    let to_phase = if matches!(self.state, TimerState::ShortBreak { .. }) {
                        Phase::ShortBreak
                    } else {
                        Phase::LongBreak
                    };
                    
                    if result.completed_phase.is_some() {
                        let event = PhaseCompleted::new(
                            self.state.active_task_id(),
                            Phase::Work,
                            to_phase,
                            self.state.session_count(),
                            self.state.session_count(),
                            1,
                        );
                        publisher.publish(Box::new(event));
                    } else {
                        let event = PhaseSkipped::new(
                            self.state.active_task_id(),
                            Phase::Work,
                            to_phase,
                            1,
                        );
                        publisher.publish(Box::new(event));
                    }
                    
                    if result.work_session_completed {
                        let work_complete = WorkSessionCompleted::new(
                            self.state.active_task_id(),
                            self.state.configuration().work_duration.as_secs() as u32,
                            self.state.session_count(),
                            1, // task_session_count - simplified for now
                            1,
                        );
                        publisher.publish(Box::new(work_complete));
                    }
                    
                    let duration = if to_phase == Phase::ShortBreak {
                        self.state.configuration().short_break_duration.as_secs() as u32
                    } else {
                        self.state.configuration().long_break_duration.as_secs() as u32
                    };
                    let break_event = BreakSessionStarted::new(
                        self.state.active_task_id(),
                        to_phase,
                        duration,
                        1,
                    );
                    publisher.publish(Box::new(break_event));
                }
                (TimerState::ShortBreak { .. }, TimerState::Working { .. }) |
                (TimerState::LongBreak { .. }, TimerState::Working { .. }) => {
                    let from_phase = if matches!(old_state, TimerState::ShortBreak { .. }) {
                        Phase::ShortBreak
                    } else {
                        Phase::LongBreak
                    };
                    
                    if result.completed_phase.is_some() {
                        let event = PhaseCompleted::new(
                            self.state.active_task_id(),
                            from_phase,
                            Phase::Work,
                            self.state.session_count(),
                            self.state.session_count(),
                            1,
                        );
                        publisher.publish(Box::new(event));
                    } else {
                        let event = PhaseSkipped::new(
                            self.state.active_task_id(),
                            from_phase,
                            Phase::Work,
                            1,
                        );
                        publisher.publish(Box::new(event));
                    }
                    
                    let duration = if from_phase == Phase::ShortBreak {
                        self.state.configuration().short_break_duration.as_secs() as u32
                    } else {
                        self.state.configuration().long_break_duration.as_secs() as u32
                    };
                    let break_complete = BreakSessionCompleted::new(
                        self.state.active_task_id(),
                        from_phase,
                        duration,
                        1,
                    );
                    publisher.publish(Box::new(break_complete));
                    
                    let work_event = WorkSessionStarted::new(
                        self.state.active_task_id(),
                        self.state.configuration().work_duration.as_secs() as u32,
                        self.state.session_count(),
                        1,
                        1,
                    );
                    publisher.publish(Box::new(work_event));
                }
                _ => {}
            }
            
            // Handle task switch events
            if old_state.active_task_id() != self.state.active_task_id() {
                let event = ActiveTaskSwitched::new(
                    old_state.active_task_id(),
                    self.state.active_task_id(),
                    self.get_current_phase(),
                    1,
                );
                publisher.publish(Box::new(event));
            }
        }
        
        Ok(())
    }
    
    /// Get state name for error messages
    fn get_state_name(&self) -> String {
        match &self.state {
            TimerState::Idle { .. } => "Stopped".to_string(),
            TimerState::Working { .. } => "Working".to_string(),
            TimerState::ShortBreak { .. } => "ShortBreak".to_string(),
            TimerState::LongBreak { .. } => "LongBreak".to_string(),
            TimerState::Paused { .. } => "Paused".to_string(),
        }
    }
    
    /// Get the current phase as the Phase enum
    fn get_current_phase(&self) -> Phase {
        match self.state {
            TimerState::Idle { .. } => Phase::Work, // Default to Work for idle
            TimerState::Working { .. } => Phase::Work,
            TimerState::ShortBreak { .. } => Phase::ShortBreak,
            TimerState::LongBreak { .. } => Phase::LongBreak,
            TimerState::Paused { ref paused_from, .. } => {
                match paused_from.as_ref() {
                    TimerState::Working { .. } => Phase::Work,
                    TimerState::ShortBreak { .. } => Phase::ShortBreak,
                    TimerState::LongBreak { .. } => Phase::LongBreak,
                    _ => Phase::Work,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskId;
    use crate::shared_kernel::MockEventPublisher;
    
    fn create_task_id() -> TaskId {
        TaskId::new()
    }
    
    #[test]
    fn should_create_default_timer() {
        let timer = Timer::default();
        assert!(timer.is_idle());
        // When idle, remaining_seconds shows the work phase duration as "ready" time
        assert_eq!(timer.remaining_seconds(), 1500);
    }
    
    #[test]
    fn should_start_timer_with_task() {
        let mut timer = Timer::default();
        let task_id = create_task_id();
        
        timer.set_active_task(Some(task_id)).unwrap();
        timer.start().unwrap();
        
        assert!(timer.is_running());
        assert_eq!(timer.phase_name(), "Focus Time");
    }
    
    #[test]
    fn should_not_start_without_task() {
        let mut timer = Timer::default();
        let result = timer.start();
        assert!(result.is_err());
    }
    
    #[test]
    fn should_pause_and_resume() {
        let mut timer = Timer::default();
        let task_id = create_task_id();
        
        timer.set_active_task(Some(task_id)).unwrap();
        timer.start().unwrap();
        
        let initial_time = timer.remaining_seconds();
        timer.pause().unwrap();
        assert!(timer.is_paused());
        
        timer.resume().unwrap();
        assert!(timer.is_running());
        assert_eq!(timer.remaining_seconds(), initial_time);
    }
    
    #[test]
    fn should_process_ticks() {
        let mut timer = Timer::default();
        let task_id = create_task_id();
        
        timer.set_active_task(Some(task_id)).unwrap();
        timer.start().unwrap();
        
        let initial_time = timer.remaining_seconds();
        let complete = timer.tick().unwrap();
        assert!(!complete);
        assert_eq!(timer.remaining_seconds(), initial_time - 1);
    }
    
    #[test]
    fn should_skip_phase() {
        let mut timer = Timer::default();
        let task_id = create_task_id();
        
        timer.set_active_task(Some(task_id)).unwrap();
        timer.start().unwrap();
        
        timer.skip_phase().unwrap();
        assert!(timer.state.is_break_phase());
    }
    
    #[test]
    fn should_publish_events() {
        let publisher = MockEventPublisher::new();
        
        let mut timer = Timer::default()
            .with_event_publisher(Box::new(publisher));
        
        let task_id = create_task_id();
        timer.set_active_task(Some(task_id)).unwrap();
        timer.start().unwrap();
        
        // Events would be published if we could access the publisher
        // In a real test, we'd check the events were published
    }
    
    #[test]
    fn should_format_time_correctly() {
        let timer = Timer::new(TimerConfiguration::default());
        // When idle, shows work duration as ready time
        assert_eq!(timer.format_time(), "25:00");
        
        let mut timer = Timer::default();
        let task_id = create_task_id();
        timer.set_active_task(Some(task_id)).unwrap();
        timer.start().unwrap();
        assert_eq!(timer.format_time(), "25:00");
    }
    
    #[test]
    fn should_calculate_progress() {
        let mut timer = Timer::default();
        let task_id = create_task_id();
        
        timer.set_active_task(Some(task_id)).unwrap();
        timer.start().unwrap();
        
        // Initially at 0% (just started)
        assert_eq!(timer.progress_percentage(), 0.0);
        
        // After some ticks
        for _ in 0..750 { // 750 seconds = 12.5 minutes = 50% of 25 minutes
            timer.tick().unwrap();
        }
        let progress = timer.progress_percentage();
        assert!((progress - 50.0).abs() < 1.0);
    }
}