use domain::{
    TimerState, PhaseTransitionService, 
    Result, Error, TimerStatus
};
use std::sync::Arc;

pub async fn pause_session(
    timer_state: &mut TimerState,
    phase_service: &Arc<dyn PhaseTransitionService + Send + Sync>,
) -> Result<()> {
    if timer_state.status() != TimerStatus::Running {
        return Err(Error::InvalidStateTransition {
            from: format!("{:?}", timer_state.status()),
            to: "Paused".to_string(),
        });
    }
    
    if timer_state.active_task_id.is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "pause_session".to_string(),
        });
    }
    
    phase_service.pause_timer(timer_state)?;
    
    Ok(())
}

pub async fn resume_session(
    timer_state: &mut TimerState,
    phase_service: &Arc<dyn PhaseTransitionService + Send + Sync>,
) -> Result<()> {
    if timer_state.status() != TimerStatus::Paused {
        return Err(Error::InvalidStateTransition {
            from: format!("{:?}", timer_state.status()),
            to: "Running".to_string(),
        });
    }
    
    if timer_state.active_task_id.is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "resume_session".to_string(),
        });
    }
    
    phase_service.start_timer(timer_state)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        TaskId, DefaultPhaseTransitionService, TimerStatus
    };

    fn setup() -> (
        Arc<dyn PhaseTransitionService + Send + Sync>,
        TaskId,
    ) {
        let phase_service: Arc<dyn PhaseTransitionService + Send + Sync> = Arc::new(DefaultPhaseTransitionService::new());
        let task_id = TaskId::new();
        
        (phase_service, task_id)
    }

    #[tokio::test]
    async fn should_pause_running_session() {
        let (phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.set_status(TimerStatus::Running).unwrap();
        
        pause_session(
            &mut timer_state,
            &phase_service,
        ).await.unwrap();
        
        assert_eq!(timer_state.status(), TimerStatus::Paused);
    }

    #[tokio::test]
    async fn should_fail_to_pause_stopped_session() {
        let (phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.set_status(TimerStatus::Stopped).unwrap();
        
        let result = pause_session(
            &mut timer_state,
            &phase_service,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_to_pause_without_active_task() {
        let (phase_service, _) = setup();
        let mut timer_state = TimerState::default();
        timer_state.set_status(TimerStatus::Running).unwrap();
        
        let result = pause_session(
            &mut timer_state,
            &phase_service,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_resume_paused_session() {
        let (phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.set_status(TimerStatus::Running).unwrap();
        timer_state.set_status(TimerStatus::Paused).unwrap();
        
        resume_session(
            &mut timer_state,
            &phase_service,
        ).await.unwrap();
        
        assert_eq!(timer_state.status(), TimerStatus::Running);
    }

    #[tokio::test]
    async fn should_fail_to_resume_running_session() {
        let (phase_service, task_id) = setup();
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task_id);
        timer_state.set_status(TimerStatus::Running).unwrap();
        
        let result = resume_session(
            &mut timer_state,
            &phase_service,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_to_resume_without_active_task() {
        let (phase_service, _) = setup();
        let mut timer_state = TimerState::default();
        timer_state.set_status(TimerStatus::Running).unwrap();
        timer_state.set_status(TimerStatus::Paused).unwrap();
        
        let result = resume_session(
            &mut timer_state,
            &phase_service,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }
}