use crate::{
    Task, TimerState, Phase, DomainEventBus, PhaseCompleted, 
    TimerStarted, TimerPaused, TimerReset, Result, Error
};
use std::sync::Arc;

pub trait PhaseTransitionService: Send + Sync {
    fn transition_to_next_phase(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<PhaseTransitionResult>;
    fn start_timer(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<()>;
    fn pause_timer(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<()>;
    fn reset_timer(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<()>;
    fn can_transition(&self, timer_state: &TimerState) -> bool;
}

#[derive(Debug, Clone)]
pub struct PhaseTransitionResult {
    pub old_phase: Phase,
    pub new_phase: Phase,
    pub work_session_completed: bool,
    pub cycle_completed: bool,
}

pub struct DefaultPhaseTransitionService {
    event_bus: Arc<DomainEventBus>,
}

impl DefaultPhaseTransitionService {
    pub fn new(event_bus: Arc<DomainEventBus>) -> Self {
        Self { event_bus }
    }

    fn publish_phase_completed(&self, timer_state: &TimerState, result: &PhaseTransitionResult) {
        let event = PhaseCompleted {
            active_task_id: timer_state.active_task_id,
            completed_phase: result.old_phase.clone(),
            next_phase: result.new_phase.clone(),
            session_count: timer_state.session_count,
            task_session_count: timer_state.task_session_count,
        };

        let aggregate_id = timer_state.active_task_id
            .map(|id| format!("task-{}", id))
            .unwrap_or_else(|| "timer".to_string());

        self.event_bus.publish_typed(
            aggregate_id,
            event,
            timer_state.session_count as u64,
        );
    }

    fn publish_timer_event<T: crate::DomainEventData>(&self, timer_state: &TimerState, event_data: T) {
        let aggregate_id = timer_state.active_task_id
            .map(|id| format!("task-{}", id))
            .unwrap_or_else(|| "timer".to_string());

        self.event_bus.publish_typed(
            aggregate_id,
            event_data,
            timer_state.session_count as u64,
        );
    }
}

impl PhaseTransitionService for DefaultPhaseTransitionService {
    fn transition_to_next_phase(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<PhaseTransitionResult> {
        if !self.can_transition(timer_state) {
            return Err(Error::InvalidStateTransition {
                from: format!("{:?}", timer_state.status),
                to: "NextPhase".to_string(),
            });
        }

        let old_phase = timer_state.phase.clone();
        let (old_phase_returned, new_phase) = timer_state.next_phase(task)?;

        let work_session_completed = matches!(old_phase, Phase::Work);
        let sessions_until_long_break = task
            .map(|t| t.config.sessions_until_long_break)
            .unwrap_or(4);
        
        let cycle_completed = work_session_completed && 
            timer_state.session_count % sessions_until_long_break as u32 == 0;

        let result = PhaseTransitionResult {
            old_phase: old_phase_returned,
            new_phase,
            work_session_completed,
            cycle_completed,
        };

        self.publish_phase_completed(timer_state, &result);

        Ok(result)
    }

    fn start_timer(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<()> {
        timer_state.set_status(crate::TimerStatus::Running)?;

        let event = TimerStarted {
            active_task_id: timer_state.active_task_id,
            phase: timer_state.phase.clone(),
            duration_seconds: timer_state.get_phase_duration(task),
        };

        self.publish_timer_event(timer_state, event);
        Ok(())
    }

    fn pause_timer(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<()> {
        timer_state.set_status(crate::TimerStatus::Paused)?;

        let event = TimerPaused {
            active_task_id: timer_state.active_task_id,
            phase: timer_state.phase.clone(),
            remaining_seconds: timer_state.remaining_seconds,
        };

        self.publish_timer_event(timer_state, event);
        Ok(())
    }

    fn reset_timer(&self, timer_state: &mut TimerState, task: Option<&Task>) -> Result<()> {
        timer_state.reset_current_phase(task);

        let event = TimerReset {
            active_task_id: timer_state.active_task_id,
            phase: timer_state.phase.clone(),
        };

        self.publish_timer_event(timer_state, event);
        Ok(())
    }

    fn can_transition(&self, timer_state: &TimerState) -> bool {
        timer_state.remaining_seconds == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskConfig, TimerStatus};
    use std::time::Duration;

    fn setup_service() -> DefaultPhaseTransitionService {
        let event_bus = Arc::new(DomainEventBus::new());
        DefaultPhaseTransitionService::new(event_bus)
    }

    fn create_test_task() -> Task {
        Task::new("Test Task".to_string(), 4).unwrap()
            .with_config(TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            }).unwrap()
    }

    #[test]
    fn should_transition_from_work_to_short_break() {
        let service = setup_service();
        let task = create_test_task();
        let mut timer_state = TimerState {
            status: TimerStatus::Stopped,
            phase: Phase::Work,
            remaining_seconds: 0, // Phase completed
            session_count: 0,
            is_break_cycle: false,
            active_task_id: Some(task.id),
            task_session_count: 0,
        };

        let result = service.transition_to_next_phase(&mut timer_state, Some(&task)).unwrap();

        assert_eq!(result.old_phase, Phase::Work);
        assert_eq!(result.new_phase, Phase::ShortBreak);
        assert_eq!(timer_state.phase, Phase::ShortBreak);
        assert!(result.work_session_completed);
        assert!(!result.cycle_completed);
        assert_eq!(timer_state.session_count, 1);
    }

    #[test]
    fn should_transition_from_work_to_long_break_after_cycle() {
        let service = setup_service();
        let task = create_test_task();
        let mut timer_state = TimerState {
            status: TimerStatus::Stopped,
            phase: Phase::Work,
            remaining_seconds: 0,
            session_count: 3, // 4th session completing
            is_break_cycle: false,
            active_task_id: Some(task.id),
            task_session_count: 3,
        };

        let result = service.transition_to_next_phase(&mut timer_state, Some(&task)).unwrap();

        assert_eq!(result.old_phase, Phase::Work);
        assert_eq!(result.new_phase, Phase::LongBreak);
        assert!(result.work_session_completed);
        assert!(result.cycle_completed);
    }

    #[test]
    fn should_transition_from_break_to_work() {
        let service = setup_service();
        let task = create_test_task();
        let mut timer_state = TimerState {
            status: TimerStatus::Stopped,
            phase: Phase::ShortBreak,
            remaining_seconds: 0,
            session_count: 1,
            is_break_cycle: true,
            active_task_id: Some(task.id),
            task_session_count: 1,
        };

        let result = service.transition_to_next_phase(&mut timer_state, Some(&task)).unwrap();

        assert_eq!(result.old_phase, Phase::ShortBreak);
        assert_eq!(result.new_phase, Phase::Work);
        assert!(!result.work_session_completed);
        assert!(!timer_state.is_break_cycle);
    }

    #[test]
    fn should_start_timer() {
        let service = setup_service();
        let task = create_test_task();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task.id);

        service.start_timer(&mut timer_state, Some(&task)).unwrap();

        assert_eq!(timer_state.status, TimerStatus::Running);
    }

    #[test]
    fn should_pause_timer() {
        let service = setup_service();
        let task = create_test_task();
        let mut timer_state = TimerState::default();
        timer_state.status = TimerStatus::Running;
        timer_state.active_task_id = Some(task.id);

        service.pause_timer(&mut timer_state, Some(&task)).unwrap();

        assert_eq!(timer_state.status, TimerStatus::Paused);
    }

    #[test]
    fn should_reset_timer() {
        let service = setup_service();
        let task = create_test_task();
        let mut timer_state = TimerState::default();
        timer_state.remaining_seconds = 500;
        timer_state.active_task_id = Some(task.id);

        service.reset_timer(&mut timer_state, Some(&task)).unwrap();

        assert_eq!(timer_state.status, TimerStatus::Stopped);
        assert_eq!(timer_state.remaining_seconds, 1500); // Reset to work duration
    }

    #[test]
    fn should_not_allow_transition_when_time_remaining() {
        let service = setup_service();
        let mut timer_state = TimerState::default();
        timer_state.remaining_seconds = 100; // Time still remaining

        assert!(!service.can_transition(&timer_state));
    }

    #[test]
    fn should_allow_transition_when_time_is_zero() {
        let service = setup_service();
        let mut timer_state = TimerState::default();
        timer_state.remaining_seconds = 0;

        assert!(service.can_transition(&timer_state));
    }
}