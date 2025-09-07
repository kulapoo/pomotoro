use chrono::{DateTime, Utc};
use domain::{Result, TimerConfiguration, TimerState};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimerConfigurationDto {
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
    pub sessions_until_long_break: u8,
}

impl From<TimerConfiguration> for TimerConfigurationDto {
    fn from(config: TimerConfiguration) -> Self {
        Self {
            work_duration: config.work_duration.as_secs() as u32,
            short_break_duration: config.short_break_duration.as_secs() as u32,
            long_break_duration: config.long_break_duration.as_secs() as u32,
            sessions_until_long_break: config.sessions_until_long_break,
        }
    }
}

impl TryFrom<TimerConfigurationDto> for TimerConfiguration {
    type Error = domain::Error;

    fn try_from(dto: TimerConfigurationDto) -> Result<Self> {
        TimerConfiguration::new(
            Duration::from_secs(dto.work_duration as u64),
            Duration::from_secs(dto.short_break_duration as u64),
            Duration::from_secs(dto.long_break_duration as u64),
            dto.sessions_until_long_break,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistoryDto {
    pub task_id: String,
    pub task_name: String,
    pub phase: String,
    pub duration_seconds: u32,
    pub completed_at: DateTime<Utc>,
    pub was_skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state_type")]
pub enum TimerStateDto {
    Idle {
        configuration: TimerConfigurationDto,
        session_count: u32,
        active_entity: Option<String>,
    },
    Working {
        configuration: TimerConfigurationDto,
        remaining_seconds: u32,
        session_count: u32,
        active_entity: Option<String>,
        entity_session_count: u32,
    },
    ShortBreak {
        configuration: TimerConfigurationDto,
        remaining_seconds: u32,
        session_count: u32,
        active_entity: Option<String>,
        entity_session_count: u32,
    },
    LongBreak {
        configuration: TimerConfigurationDto,
        remaining_seconds: u32,
        session_count: u32,
        active_entity: Option<String>,
        entity_session_count: u32,
    },
    Paused {
        paused_from: Box<TimerStateDto>,
        remaining_seconds: u32,
    },
}

impl From<&TimerState> for TimerStateDto {
    fn from(state: &TimerState) -> Self {
        match state {
            TimerState::Idle => TimerStateDto::Idle {
                configuration: Default::default(),
                session_count: 0,
                active_entity: None,
            },
            TimerState::Working { remaining_seconds } => {
                TimerStateDto::Working {
                    configuration: Default::default(),
                    remaining_seconds: *remaining_seconds,
                    session_count: 0,
                    active_entity: None,
                    entity_session_count: 0,
                }
            }
            TimerState::ShortBreak { remaining_seconds } => {
                TimerStateDto::ShortBreak {
                    configuration: Default::default(),
                    remaining_seconds: *remaining_seconds,
                    session_count: 0,
                    active_entity: None,
                    entity_session_count: 0,
                }
            }
            TimerState::LongBreak { remaining_seconds } => {
                TimerStateDto::LongBreak {
                    configuration: Default::default(),
                    remaining_seconds: *remaining_seconds,
                    session_count: 0,
                    active_entity: None,
                    entity_session_count: 0,
                }
            }
            TimerState::Paused {
                paused_from,
                remaining_seconds,
            } => TimerStateDto::Paused {
                paused_from: Box::new(paused_from.as_ref().into()),
                remaining_seconds: *remaining_seconds,
            },
        }
    }
}

impl TryFrom<TimerStateDto> for TimerState {
    type Error = domain::Error;

    fn try_from(dto: TimerStateDto) -> Result<Self> {
        let state = match dto {
            TimerStateDto::Idle { .. } => TimerState::Idle,
            TimerStateDto::Working {
                remaining_seconds, ..
            } => TimerState::Working { remaining_seconds },
            TimerStateDto::ShortBreak {
                remaining_seconds, ..
            } => TimerState::ShortBreak { remaining_seconds },
            TimerStateDto::LongBreak {
                remaining_seconds, ..
            } => TimerState::LongBreak { remaining_seconds },
            TimerStateDto::Paused {
                paused_from,
                remaining_seconds,
            } => TimerState::Paused {
                paused_from: Box::new((*paused_from).try_into()?),
                remaining_seconds,
            },
        };
        Ok(state)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerDataDto {
    pub state: TimerStateDto,
    pub last_saved: DateTime<Utc>,
    pub session_history: Vec<SessionHistoryDto>,
}
