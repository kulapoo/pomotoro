use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent {
    pub fn new(
        aggregate_id: String,
        event_type: String,
        event_data: serde_json::Value,
        version: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            event_type,
            event_data,
            occurred_at: Utc::now(),
            version,
        }
    }
}

pub trait DomainEventData: Serialize + for<'de> Deserialize<'de> + Clone + std::fmt::Debug {
    fn event_type() -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_create_domain_event_with_correct_fields() {
        let aggregate_id = "task-123".to_string();
        let event_type = "TaskCreated".to_string();
        let event_data = json!({"name": "Test Task"});
        let version = 1;

        let event = DomainEvent::new(aggregate_id.clone(), event_type.clone(), event_data.clone(), version);

        assert_eq!(event.aggregate_id, aggregate_id);
        assert_eq!(event.event_type, event_type);
        assert_eq!(event.event_data, event_data);
        assert_eq!(event.version, version);
        assert!(event.occurred_at <= Utc::now());
        assert!(!event.id.is_nil());
    }

    #[test]
    fn should_serialize_and_deserialize_domain_event() {
        let event = DomainEvent::new(
            "test-123".to_string(),
            "TestEvent".to_string(),
            json!({"test": "data"}),
            1,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: DomainEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}