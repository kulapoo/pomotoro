pub mod active_task_switched;
pub mod break_session_completed;
pub mod break_session_started;
pub mod phase_completed;
pub mod phase_skipped;
pub mod session_flow_reset;
pub mod timer_paused;
pub mod timer_reset;
pub mod timer_started;
pub mod timer_status_changed;
pub mod timer_tick;
pub mod work_session_completed;
pub mod work_session_started;

pub use active_task_switched::ActiveTaskSwitched;
pub use break_session_completed::BreakSessionCompleted;
pub use break_session_started::BreakSessionStarted;
pub use phase_completed::PhaseCompleted;
pub use phase_skipped::PhaseSkipped;
pub use session_flow_reset::SessionFlowReset;
pub use timer_paused::Paused;
pub use timer_reset::Reset;
pub use timer_started::Started;
pub use timer_status_changed::StatusChanged;
pub use timer_tick::Tick;
pub use work_session_completed::WorkSessionCompleted;
pub use work_session_started::WorkSessionStarted;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Event;
    use crate::timer::Phase;
    #[test]
    fn should_have_correct_event_types() {
        let timer_started = Started::new(
            Some(uuid::Uuid::new_v4().to_string()),
            Phase::Work,
            1500,
            1,
        );
        let timer_paused = Paused::new(
            Some(uuid::Uuid::new_v4().to_string()),
            Phase::Work,
            1200,
            2,
        );

        assert_eq!(timer_started.event_type(), "Started");
        assert_eq!(timer_paused.event_type(), "Paused");
        assert_eq!(timer_started.version(), 1);
        assert_eq!(timer_paused.version(), 2);
    }

    #[test]
    fn should_serialize_timer_started_event() {
        let event = Started::new(
            Some(uuid::Uuid::new_v4().to_string()),
            Phase::Work,
            1500,
            1,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: Started = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_phase_completed_event() {
        let event = PhaseCompleted::new(
            Some(uuid::Uuid::new_v4().to_string()),
            Phase::Work,
            Phase::ShortBreak,
            1,
            1,
            2,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: PhaseCompleted =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_active_task_switched_event() {
        let old_entity_id = uuid::Uuid::new_v4().to_string();
        let new_entity_id = uuid::Uuid::new_v4().to_string();

        let event = ActiveTaskSwitched::new(
            Some(old_entity_id),
            Some(new_entity_id),
            Phase::Work,
            3,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ActiveTaskSwitched =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}
