use pomotoro_domain::{
    TimerState, PhaseTransitionService, EventPublisher, 
    Result, Error, TimerStatus
};
use std::sync::Arc;

pub async fn reset_session(
    timer_state: &mut TimerState,
    phase_service: &Arc<dyn PhaseTransitionService + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    // Ensure we have an active task
    if timer_state.active_task_id.is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "reset_session".to_string(),
        });
    }
    
    // Cannot reset while running - must pause or stop first
    if timer_state.status() == TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Reset".to_string(),
        });
    }
    
    // Reset the timer using the phase transition service
    phase_service.reset_timer(timer_state)?;
    
    Ok(())
}

pub async fn reset_full_session(
    timer_state: &mut TimerState,
    phase_service: &Arc<dyn PhaseTransitionService + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    // Ensure we have an active task
    if timer_state.active_task_id.is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "reset_full_session".to_string(),
        });
    }
    
    // Cannot reset while running
    if timer_state.status() == TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "FullReset".to_string(),
        });
    }
    
    // Reset task session count and timer state
    timer_state.task_session_count = 0;
    
    // Reset timer core state
    timer_state.timer.session_count = 0;
    timer_state.timer.is_break_cycle = false;
    timer_state.timer.phase = pomotoro_domain::Phase::Work;
    
    // Reset current phase using the service
    phase_service.reset_timer(timer_state)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{
        TaskId, NoOpEventPublisher, DefaultPhaseTransitionService, 
        TimerStatus, Phase
    };

    fn setup() -> (
        Arc<dyn EventPublisher + Send + Sync>,
        Arc<dyn PhaseTransitionService + Send + Sync>,
        TaskId,
    ) {
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        let phase_service: Arc<dyn PhaseTransitionService + Send + Sync> = Arc::new(DefaultPhaseTransitionService::new());
        let task_id = TaskId::new();
        
        (event_publisher, phase_service, task_id)
    }

    #[tokio::test]
    async fn should_reset_current_session() {
        let (event_publisher, phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.timer.remaining_seconds = 500; // Partially completed
        timer_state.set_status(TimerStatus::Running).unwrap(); // First go to Running
        timer_state.set_status(TimerStatus::Paused).unwrap(); // Then can pause
        
        reset_session(
            &mut timer_state,
            &phase_service,
            &event_publisher,
        ).await.unwrap();
        
        assert_eq!(timer_state.status(), TimerStatus::Stopped);
        // Should reset to full work duration (1500 seconds)
        assert_eq!(timer_state.remaining_seconds(), 1500);
    }

    #[tokio::test]
    async fn should_fail_to_reset_running_session() {
        let (event_publisher, phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.set_status(TimerStatus::Running).unwrap();
        
        let result = reset_session(
            &mut timer_state,
            &phase_service,
            &event_publisher,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_to_reset_without_active_task() {
        let (event_publisher, phase_service, _) = setup();
        let mut timer_state = TimerState::default();
        timer_state.set_status(TimerStatus::Stopped).unwrap();
        
        let result = reset_session(
            &mut timer_state,
            &phase_service,
            &event_publisher,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_reset_full_session() {
        let (event_publisher, phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.task_session_count = 3;
        timer_state.timer.session_count = 2;
        timer_state.timer.is_break_cycle = true;
        timer_state.timer.phase = Phase::ShortBreak;
        timer_state.set_status(TimerStatus::Stopped).unwrap();
        
        reset_full_session(
            &mut timer_state,
            &phase_service,
            &event_publisher,
        ).await.unwrap();
        
        assert_eq!(timer_state.status(), TimerStatus::Stopped);
        assert_eq!(timer_state.task_session_count, 0);
        assert_eq!(timer_state.timer.session_count, 0);
        assert!(!timer_state.timer.is_break_cycle);
        assert_eq!(timer_state.timer.phase, Phase::Work);
        assert_eq!(timer_state.remaining_seconds(), 1500); // Work duration
    }

    #[tokio::test]
    async fn should_fail_to_full_reset_running_session() {
        let (event_publisher, phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.set_status(TimerStatus::Running).unwrap();
        
        let result = reset_full_session(
            &mut timer_state,
            &phase_service,
            &event_publisher,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_to_full_reset_without_active_task() {
        let (event_publisher, phase_service, _) = setup();
        let mut timer_state = TimerState::default();
        timer_state.set_status(TimerStatus::Stopped).unwrap();
        
        let result = reset_full_session(
            &mut timer_state,
            &phase_service,
            &event_publisher,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }
}