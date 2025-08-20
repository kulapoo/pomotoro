use crate::Event;
use crate::task::id::Id as TaskId;
use crate::timer::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tick {
    pub task_id: Option<TaskId>,
    pub phase: Phase,
    pub remaining_seconds: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl Tick {
    pub fn new(
        task_id: Option<TaskId>,
        phase: Phase,
        remaining_seconds: u32,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            phase,
            remaining_seconds,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl Event for Tick {
    fn event_type(&self) -> &'static str {
        "Tick"
    }

    fn aggregate_id(&self) -> String {
        self.task_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_timer_tick_event() {
        let task_id = TaskId::new();
        let event = Tick::new(Some(task_id), Phase::Work, 1234, 1);

        assert_eq!(event.event_type(), "Tick");
        assert_eq!(event.version(), 1);
        assert_eq!(event.remaining_seconds, 1234);
        assert_eq!(event.phase, Phase::Work);
    }

    #[test]
    fn should_serialize_timer_tick_event() {
        let event = Tick::new(Some(TaskId::new()), Phase::ShortBreak, 300, 2);

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: Tick = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}
