use std::sync::Arc;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::upsert::excluded;
use chrono::Utc;
use domain::{Error, Result, TimerState, TimerConfiguration};
use serde::{Deserialize, Serialize};

use crate::schema::timer_state;
use super::{DbPool, models::TimerStateDb};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredTimerState {
    pub state_data: String, // JSON serialized TimerState
    pub session_history: Vec<SessionEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SessionEntry {
    pub task_id: String,
    pub session_type: String,
    pub duration: u32,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait TimerRepository: Send + Sync {
    async fn save_timer_state(&self, timer: &domain::timer::Timer) -> Result<()>;
    async fn load_timer_state(&self) -> Result<Option<domain::timer::Timer>>;
    async fn clear_timer_state(&self) -> Result<()>;
}

pub struct SqliteTimerRepository {
    pool: Arc<DbPool>,
}

impl SqliteTimerRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TimerRepository for SqliteTimerRepository {
    async fn save_timer_state(&self, timer: &domain::timer::Timer) -> Result<()> {
        let pool = self.pool.clone();
        
        // We'll store the timer state in a structured way
        // For now we're just storing the configuration and state separately
        
        // Extract current state information
        let state = timer.state();
        let (current_phase, remaining_seconds, is_running, current_task_id, session_count) = match state {
            TimerState::Idle { session_count, active_entity, .. } => {
                ("idle".to_string(), 0, false, active_entity.clone(), *session_count)
            },
            TimerState::Working { remaining_seconds, session_count, active_entity, .. } => {
                ("working".to_string(), *remaining_seconds, true, active_entity.clone(), *session_count)
            },
            TimerState::ShortBreak { remaining_seconds, session_count, active_entity, .. } => {
                ("short_break".to_string(), *remaining_seconds, true, active_entity.clone(), *session_count)
            },
            TimerState::LongBreak { remaining_seconds, session_count, active_entity, .. } => {
                ("long_break".to_string(), *remaining_seconds, true, active_entity.clone(), *session_count)
            },
            TimerState::Paused { paused_from, remaining_seconds } => {
                let phase = match paused_from.as_ref() {
                    TimerState::Working { .. } => "working_paused",
                    TimerState::ShortBreak { .. } => "short_break_paused",
                    TimerState::LongBreak { .. } => "long_break_paused",
                    _ => "paused",
                };
                let active = paused_from.active_entity().map(|s| s.to_string());
                let count = paused_from.session_count();
                (phase.to_string(), *remaining_seconds, false, active, count)
            },
        };
        
        // Get configuration as JSON
        let config_json = serde_json::to_string(state.configuration())
            .map_err(|e| Error::SerializationError {
                message: format!("Failed to serialize timer configuration: {}", e),
            })?;
        
        let now = Utc::now().to_rfc3339();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Error::RepositoryError {
                message: format!("Failed to get database connection: {}", e),
            })?;
            
            // Use upsert pattern for single-row table
            diesel::insert_into(timer_state::table)
                .values(TimerStateDb {
                    id: 1, // Always use ID 1 for single-row table
                    timer_config: config_json,
                    current_phase,
                    remaining_seconds: remaining_seconds as i32,
                    is_running,
                    current_task_id,
                    session_count: session_count as i32,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                })
                .on_conflict(timer_state::dsl::id)
                .do_update()
                .set((
                    timer_state::dsl::timer_config.eq(excluded(timer_state::dsl::timer_config)),
                    timer_state::dsl::current_phase.eq(excluded(timer_state::dsl::current_phase)),
                    timer_state::dsl::remaining_seconds.eq(excluded(timer_state::dsl::remaining_seconds)),
                    timer_state::dsl::is_running.eq(excluded(timer_state::dsl::is_running)),
                    timer_state::dsl::current_task_id.eq(excluded(timer_state::dsl::current_task_id)),
                    timer_state::dsl::session_count.eq(excluded(timer_state::dsl::session_count)),
                    timer_state::dsl::updated_at.eq(excluded(timer_state::dsl::updated_at)),
                ))
                .execute(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to save timer state: {}", e),
                })?;
            
            Ok(())
        })
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Task join error: {}", e),
        })?
    }
    
    async fn load_timer_state(&self) -> Result<Option<domain::timer::Timer>> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Error::RepositoryError {
                message: format!("Failed to get database connection: {}", e),
            })?;
            
            let result: Option<TimerStateDb> = timer_state::table
                .filter(timer_state::id.eq(1))
                .first(&mut conn)
                .optional()
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to load timer state: {}", e),
                })?;
            
            match result {
                Some(db_state) => {
                    // Try to reconstruct the timer from the stored state
                    // For simplicity, we'll create a new timer with the stored configuration
                    // and manually set its state based on the stored phase
                    
                    let config: TimerConfiguration = serde_json::from_str(&db_state.timer_config)
                        .map_err(|e| Error::DeserializationError {
                            message: format!("Failed to deserialize timer configuration: {}", e),
                        })?;
                    
                    let mut timer = domain::timer::Timer::new(config.clone());
                    
                    // Set the timer state based on stored phase
                    // Note: This is a simplified restoration - in a real implementation,
                    // you might want to store the complete serialized timer state
                    match db_state.current_phase.as_str() {
                        "idle" => {
                            // Timer is already in idle state by default
                        },
                        "working" | "short_break" | "long_break" => {
                            // For active states, we'd need to restore the timer
                            // This is simplified - you might want to store complete state
                            if db_state.is_running {
                                // Start the timer to move it to working state
                                timer.start().ok();
                            }
                        },
                        _ => {
                            // Handle paused states - would need more complex restoration
                        }
                    }
                    
                    Ok(Some(timer))
                },
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Task join error: {}", e),
        })?
    }
    
    async fn clear_timer_state(&self) -> Result<()> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Error::RepositoryError {
                message: format!("Failed to get database connection: {}", e),
            })?;
            
            diesel::delete(timer_state::table.filter(timer_state::id.eq(1)))
                .execute(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to clear timer state: {}", e),
                })?;
            
            Ok(())
        })
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Task join error: {}", e),
        })?
    }
}