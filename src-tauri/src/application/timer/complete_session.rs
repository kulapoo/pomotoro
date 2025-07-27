use pomotoro_domain::{
    TimerState, TaskRepository, PhaseTransitionService,
    EventPublisher, Result, Error, Phase
};
use crate::application::task::{complete_session as complete_task_session};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SessionCompleted {
    pub old_phase: Phase,
    pub new_phase: Phase,
    pub work_session_completed: bool,
    pub cycle_completed: bool,
    pub task_completed: bool,
    pub sessions_completed: u8,
    pub total_sessions: u8,
}

pub async fn complete_timer_session(
    timer_state: &mut TimerState,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    phase_service: &Arc<dyn PhaseTransitionService + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<SessionCompleted> {
    // Ensure we have an active task
    let active_task_id = timer_state.active_task_id
        .as_ref()
        .ok_or_else(|| Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "complete_session".to_string(),
        })?;
    
    // Ensure timer is ready for phase transition (time should be 0)
    if !phase_service.can_transition(timer_state) {
        return Err(Error::InvalidStateTransition {
            from: "time_remaining".to_string(),
            to: "complete_session".to_string(),
        });
    }
    
    let current_phase = timer_state.phase();
    
    // If completing a work session, increment task session count
    let task_session_result = if current_phase == Phase::Work {
        Some(complete_task_session(task_repo, event_publisher, &active_task_id.to_string()).await?)
    } else {
        None
    };
    
    // Transition to next phase
    let phase_result = phase_service.transition_to_next_phase(timer_state)?;
    
    // Extract task session information
    let (task_completed, sessions_completed, total_sessions) = if let Some(result) = task_session_result {
        (result.task_completed, result.sessions_completed, result.total_sessions)
    } else {
        (false, 0, 0)
    };
    
    Ok(SessionCompleted {
        old_phase: phase_result.old_phase,
        new_phase: phase_result.new_phase,
        work_session_completed: phase_result.work_session_completed,
        cycle_completed: phase_result.cycle_completed,
        task_completed,
        sessions_completed,
        total_sessions,
    })
}

pub async fn force_complete_timer_session(
    timer_state: &mut TimerState,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    phase_service: &Arc<dyn PhaseTransitionService + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<SessionCompleted> {
    // Ensure we have an active task
    if timer_state.active_task_id.is_none() {
        return Err(Error::InvalidStateTransition {
            from: "no_active_task".to_string(),
            to: "force_complete_session".to_string(),
        });
    }
    
    // Force timer to completion state
    timer_state.timer.remaining_seconds = 0;
    timer_state.set_status(pomotoro_domain::TimerStatus::Stopped)?;
    
    // Complete the session normally
    complete_timer_session(
        timer_state,
        task_repo,
        phase_service,
        event_publisher,
    ).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{
        Task, NoOpEventPublisher, 
        DefaultPhaseTransitionService, TimerStatus, Phase
    };
    use crate::infrastructure::InMemoryTaskRepository;

    async fn setup() -> (
        Arc<dyn TaskRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
        Arc<dyn PhaseTransitionService + Send + Sync>,
        Task,
    ) {
        let task_repo: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        let phase_service: Arc<dyn PhaseTransitionService + Send + Sync> = Arc::new(DefaultPhaseTransitionService::new());
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        task_repo.create(task.clone()).await.unwrap();
        
        (task_repo, event_publisher, phase_service, task)
    }

    #[tokio::test]
    async fn should_complete_work_session() {
        let (task_repo, event_publisher, phase_service, task) = setup().await;
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task.id.clone());
        timer_state.timer.remaining_seconds = 0; // Phase completed
        timer_state.timer.phase = Phase::Work;
        
        let result = complete_timer_session(
            &mut timer_state,
            &task_repo,
            &phase_service,
            &event_publisher,
        ).await.unwrap();
        
        assert_eq!(result.old_phase, Phase::Work);
        assert_eq!(result.new_phase, Phase::ShortBreak);
        assert!(result.work_session_completed);
        assert!(!result.cycle_completed);
        assert!(!result.task_completed);
        assert_eq!(result.sessions_completed, 1);
        assert_eq!(result.total_sessions, 4);
        
        // Verify timer transitioned to break
        assert_eq!(timer_state.phase(), Phase::ShortBreak);
    }

    #[tokio::test]
    async fn should_complete_break_session() {
        let (task_repo, event_publisher, phase_service, task) = setup().await;
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task.id.clone());
        timer_state.timer.remaining_seconds = 0;
        timer_state.timer.phase = Phase::ShortBreak;
        timer_state.timer.is_break_cycle = true;
        timer_state.timer.session_count = 1;
        
        let result = complete_timer_session(
            &mut timer_state,
            &task_repo,
            &phase_service,
            &event_publisher,
        ).await.unwrap();
        
        assert_eq!(result.old_phase, Phase::ShortBreak);
        assert_eq!(result.new_phase, Phase::Work);
        assert!(!result.work_session_completed);
        assert!(!result.task_completed);
        
        // Verify timer transitioned back to work
        assert_eq!(timer_state.phase(), Phase::Work);
        assert!(!timer_state.is_break_cycle());
    }

    #[tokio::test]
    async fn should_complete_task_on_final_session() {
        let (task_repo, event_publisher, phase_service, _) = setup().await;
        
        // Create a task with only 1 session
        let single_session_task = Task::new("Single Session Task".to_string(), 1).unwrap();
        task_repo.create(single_session_task.clone()).await.unwrap();
        
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(single_session_task.id.clone());
        timer_state.timer.remaining_seconds = 0;
        timer_state.timer.phase = Phase::Work;
        
        let result = complete_timer_session(
            &mut timer_state,
            &task_repo,
            &phase_service,
            &event_publisher,
        ).await.unwrap();
        
        assert!(result.work_session_completed);
        assert!(result.task_completed);
        assert_eq!(result.sessions_completed, 1);
        assert_eq!(result.total_sessions, 1);
    }

    #[tokio::test]
    async fn should_fail_without_active_task() {
        let (task_repo, event_publisher, phase_service, _) = setup().await;
        let mut timer_state = TimerState::default();
        timer_state.timer.remaining_seconds = 0;
        
        let result = complete_timer_session(
            &mut timer_state,
            &task_repo,
            &phase_service,
            &event_publisher,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_fail_with_time_remaining() {
        let (task_repo, event_publisher, phase_service, task) = setup().await;
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task.id.clone());
        timer_state.timer.remaining_seconds = 500; // Time still remaining
        
        let result = complete_timer_session(
            &mut timer_state,
            &task_repo,
            &phase_service,
            &event_publisher,
        ).await;
        
        assert!(matches!(result, Err(Error::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn should_force_complete_session() {
        let (task_repo, event_publisher, phase_service, task) = setup().await;
        let mut timer_state = TimerState::default();
        timer_state.active_task_id = Some(task.id.clone());
        timer_state.timer.remaining_seconds = 500; // Time remaining
        timer_state.timer.phase = Phase::Work;
        timer_state.set_status(TimerStatus::Running).unwrap();
        
        let result = force_complete_timer_session(
            &mut timer_state,
            &task_repo,
            &phase_service,
            &event_publisher,
        ).await.unwrap();
        
        assert_eq!(result.old_phase, Phase::Work);
        assert_eq!(result.new_phase, Phase::ShortBreak);
        assert!(result.work_session_completed);
        assert_eq!(timer_state.status(), TimerStatus::Stopped);
        assert_eq!(timer_state.phase(), Phase::ShortBreak);
    }
}