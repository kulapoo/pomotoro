pub mod timer_started;
pub mod timer_paused;
pub mod timer_reset;
pub mod phase_completed;
pub mod phase_skipped;
pub mod timer_status_changed;
pub mod active_task_switched;
pub mod break_session_started;
pub mod break_session_completed;
pub mod work_session_started;
pub mod work_session_completed;
pub mod session_started;
pub mod session_flow_reset;

// Re-export all timer events
pub use timer_started::TimerStarted;
pub use timer_paused::TimerPaused;
pub use timer_reset::TimerReset;
pub use phase_completed::PhaseCompleted;
pub use phase_skipped::PhaseSkipped;
pub use timer_status_changed::TimerStatusChanged;
pub use active_task_switched::ActiveTaskSwitched;
pub use break_session_started::BreakSessionStarted;
pub use break_session_completed::BreakSessionCompleted;
pub use work_session_started::WorkSessionStarted;
pub use work_session_completed::WorkSessionCompleted;
pub use session_started::SessionStarted;
pub use session_flow_reset::SessionFlowReset;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskId, Phase};
    use crate::DomainEvent;

    #[test]
    fn should_have_correct_event_types() {
        let timer_started = TimerStarted::new(Some(TaskId::new()), Phase::Work, 1500, 1);
        let timer_paused = TimerPaused::new(Some(TaskId::new()), Phase::Work, 1200, 2);
        
        assert_eq!(timer_started.event_type(), "TimerStarted");
        assert_eq!(timer_paused.event_type(), "TimerPaused");
        assert_eq!(timer_started.version(), 1);
        assert_eq!(timer_paused.version(), 2);
    }

    #[test]
    fn should_serialize_timer_started_event() {
        let event = TimerStarted::new(Some(TaskId::new()), Phase::Work, 1500, 1);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TimerStarted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_phase_completed_event() {
        let event = PhaseCompleted::new(
            Some(TaskId::new()),
            Phase::Work,
            Phase::ShortBreak,
            1,
            1,
            2,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: PhaseCompleted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_active_task_switched_event() {
        let old_task_id = TaskId::new();
        let new_task_id = TaskId::new();
        
        let event = ActiveTaskSwitched::new(Some(old_task_id), Some(new_task_id), Phase::Work, 3);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ActiveTaskSwitched = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}