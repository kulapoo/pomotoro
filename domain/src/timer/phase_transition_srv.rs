use crate::{
    TimerState, Phase, Result, Error
};

#[cfg(test)]
use crate::Timer;

pub trait PhaseTransitionService: Send + Sync {
    fn transition_to_next_phase(&self, timer_state: &mut TimerState) -> Result<PhaseTransitionResult>;
    fn start_timer(&self, timer_state: &mut TimerState) -> Result<()>;
    fn pause_timer(&self, timer_state: &mut TimerState) -> Result<()>;
    fn reset_timer(&self, timer_state: &mut TimerState) -> Result<()>;
    fn can_transition(&self, timer_state: &TimerState) -> bool;
}

#[derive(Debug, Clone)]
pub struct PhaseTransitionResult {
    pub old_phase: Phase,
    pub new_phase: Phase,
    pub work_session_completed: bool,
    pub cycle_completed: bool,
}

pub struct DefaultPhaseTransitionService;

impl Default for DefaultPhaseTransitionService {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultPhaseTransitionService {
    pub fn new() -> Self {
        Self
    }

}

impl PhaseTransitionService for DefaultPhaseTransitionService {
    fn transition_to_next_phase(&self, timer_state: &mut TimerState) -> Result<PhaseTransitionResult> {
        if !self.can_transition(timer_state) {
            return Err(Error::InvalidStateTransition {
                from: format!("{:?}", timer_state.status()),
                to: "NextPhase".to_string(),
            });
        }

        let old_phase = timer_state.phase();
        let (old_phase_returned, new_phase) = timer_state.next_phase()?;

        let work_session_completed = matches!(old_phase, Phase::Work);
        let sessions_until_long_break = timer_state.configuration.sessions_until_long_break as u32;

        let cycle_completed = work_session_completed &&
            timer_state.session_count() % sessions_until_long_break == 0;

        let result = PhaseTransitionResult {
            old_phase: old_phase_returned,
            new_phase,
            work_session_completed,
            cycle_completed,
        };

        Ok(result)
    }

    fn start_timer(&self, timer_state: &mut TimerState) -> Result<()> {
        timer_state.set_status(crate::TimerStatus::Running)?;
        Ok(())
    }

    fn pause_timer(&self, timer_state: &mut TimerState) -> Result<()> {
        timer_state.set_status(crate::TimerStatus::Paused)?;
        Ok(())
    }

    fn reset_timer(&self, timer_state: &mut TimerState) -> Result<()> {
        timer_state.reset_current_phase();
        Ok(())
    }

    fn can_transition(&self, timer_state: &TimerState) -> bool {
        timer_state.remaining_seconds() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TimerStatus, TimerConfiguration};
    use std::time::Duration;

    fn setup_service() -> DefaultPhaseTransitionService {
        DefaultPhaseTransitionService::new()
    }

    fn create_test_configuration() -> TimerConfiguration {
        TimerConfiguration {
            work_duration: Duration::from_secs(1500),
            short_break_duration: Duration::from_secs(300),
            long_break_duration: Duration::from_secs(900),
            sessions_until_long_break: 4,
        }
    }

    #[test]
    fn should_transition_from_work_to_short_break() {
        let service = setup_service();
        let configuration = create_test_configuration();
        let mut timer_state = TimerState {
            timer: Timer {
                status: TimerStatus::Stopped,
                phase: Phase::Work,
                remaining_seconds: 0, // Phase completed
                session_count: 0,
                is_break_cycle: false,
            },
            active_task_id: None,
            configuration,
            task_session_count: 0,
        };

        let result = service.transition_to_next_phase(&mut timer_state).unwrap();

        assert_eq!(result.old_phase, Phase::Work);
        assert_eq!(result.new_phase, Phase::ShortBreak);
        assert_eq!(timer_state.phase(), Phase::ShortBreak);
        assert!(result.work_session_completed);
        assert!(!result.cycle_completed);
        assert_eq!(timer_state.session_count(), 1);
    }

    #[test]
    fn should_transition_from_work_to_long_break_after_cycle() {
        let service = setup_service();
        let configuration = create_test_configuration();
        let mut timer_state = TimerState {
            timer: Timer {
                status: TimerStatus::Stopped,
                phase: Phase::Work,
                remaining_seconds: 0,
                session_count: 3, // 4th session completing
                is_break_cycle: false,
            },
            active_task_id: None,
            configuration,
            task_session_count: 3,
        };

        let result = service.transition_to_next_phase(&mut timer_state).unwrap();

        assert_eq!(result.old_phase, Phase::Work);
        assert_eq!(result.new_phase, Phase::LongBreak);
        assert!(result.work_session_completed);
        assert!(result.cycle_completed);
    }

    #[test]
    fn should_transition_from_break_to_work() {
        let service = setup_service();
        let configuration = create_test_configuration();
        let mut timer_state = TimerState {
            timer: Timer {
                status: TimerStatus::Stopped,
                phase: Phase::ShortBreak,
                remaining_seconds: 0,
                session_count: 1,
                is_break_cycle: true,
            },
            active_task_id: None,
            configuration,
            task_session_count: 1,
        };

        let result = service.transition_to_next_phase(&mut timer_state).unwrap();

        assert_eq!(result.old_phase, Phase::ShortBreak);
        assert_eq!(result.new_phase, Phase::Work);
        assert!(!result.work_session_completed);
        assert!(!timer_state.is_break_cycle());
    }

    #[test]
    fn should_start_timer() {
        let service = setup_service();
        let mut timer_state = TimerState::default();

        service.start_timer(&mut timer_state).unwrap();

        assert_eq!(timer_state.status(), TimerStatus::Running);
    }

    #[test]
    fn should_pause_timer() {
        let service = setup_service();
        let mut timer_state = TimerState::default();
        timer_state.set_status(TimerStatus::Running).unwrap();

        service.pause_timer(&mut timer_state).unwrap();

        assert_eq!(timer_state.status(), TimerStatus::Paused);
    }

    #[test]
    fn should_reset_timer() {
        let service = setup_service();
        let mut timer_state = TimerState::default();
        timer_state.timer.remaining_seconds = 500;

        service.reset_timer(&mut timer_state).unwrap();

        assert_eq!(timer_state.status(), TimerStatus::Stopped);
        assert_eq!(timer_state.remaining_seconds(), 1500); // Reset to work duration
    }

    #[test]
    fn should_not_allow_transition_when_time_remaining() {
        let service = setup_service();
        let mut timer_state = TimerState::default();
        timer_state.timer.remaining_seconds = 100; // Time still remaining

        assert!(!service.can_transition(&timer_state));
    }

    #[test]
    fn should_allow_transition_when_time_is_zero() {
        let service = setup_service();
        let mut timer_state = TimerState::default();
        timer_state.timer.remaining_seconds = 0;

        assert!(service.can_transition(&timer_state));
    }
}