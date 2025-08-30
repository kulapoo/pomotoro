use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Active,
    Queued,
    Completed,
    Paused,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, Status::Completed)
    }

    pub fn can_be_started(&self) -> bool {
        matches!(self, Status::Active | Status::Queued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_is_active() {
        assert!(Status::Active.is_active());
        assert!(!Status::Queued.is_active());
        assert!(!Status::Completed.is_active());
        assert!(!Status::Paused.is_active());
    }

    #[test]
    fn test_status_is_completed() {
        assert!(!Status::Active.is_completed());
        assert!(!Status::Queued.is_completed());
        assert!(Status::Completed.is_completed());
        assert!(!Status::Paused.is_completed());
    }

    #[test]
    fn test_status_can_be_started() {
        assert!(Status::Active.can_be_started());
        assert!(Status::Queued.can_be_started());
        assert!(!Status::Completed.can_be_started());
        assert!(!Status::Paused.can_be_started());
    }

    #[test]
    fn test_status_equality() {
        assert_eq!(Status::Active, Status::Active);
        assert_ne!(Status::Active, Status::Paused);
        assert_ne!(Status::Queued, Status::Completed);
    }

    #[test]
    fn test_status_clone() {
        let original = Status::Paused;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_status_serialization() {
        let statuses = vec![
            Status::Active,
            Status::Queued,
            Status::Completed,
            Status::Paused,
        ];

        for status in statuses {
            let serialized = serde_json::to_string(&status).unwrap();
            let deserialized: Status = serde_json::from_str(&serialized).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}
