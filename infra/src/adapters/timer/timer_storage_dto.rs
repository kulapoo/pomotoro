use super::timer_dto::SessionHistoryDto;
use chrono::{DateTime, Utc};
use domain::timer::Timer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerStorageDto {
    pub timer: Timer,
    pub last_saved: DateTime<Utc>,
    pub session_history: Vec<SessionHistoryDto>,
}
