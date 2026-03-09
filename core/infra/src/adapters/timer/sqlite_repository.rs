use crate::adapters::DbPool;
use crate::adapters::database::models::TimerDb;
use crate::schema::timers;
use async_trait::async_trait;
use diesel::prelude::*;
use domain::timer::{Error, Result};
use domain::{DEFAULT_TASK_ID, Timer, TimerRepository};
use std::sync::Arc;

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
    async fn get(&self) -> Result<Timer> {
        let mut conn = self.pool.get().map_err(|e| {
            Error::InvalidOperation(format!("Failed to get connection: {}", e))
        })?;
        let timer_id = DEFAULT_TASK_ID.as_str();

        let timer_db = timers::table
            .filter(timers::id.eq(&timer_id))
            .first::<TimerDb>(&mut conn)
            .optional()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get timer: {}", e),
            })?;

        match timer_db {
            Some(db) => {
                Timer::try_from(db).map_err(|e| Error::RepositoryError {
                    message: format!(
                        "Failed to convert timer from database: {}",
                        e
                    ),
                })
            }
            None => {
                // NOTE: Lazily create default timer on first access
                // This ensures backwards compatibility and simplifies initialization
                println!("Timer doesn't exist, creating the default one");
                let timer = Timer::default_timer();
                self.save(&timer).await?;
                Ok(timer)
            }
        }
    }

    async fn save(&self, timer: &Timer) -> Result<()> {
        let mut timer_db = TimerDb::from(timer.clone());
        // Ensure consistent row identity: always use DEFAULT_TASK_ID as the
        // primary key so that `get()` can find the row by the same key.
        timer_db.id = DEFAULT_TASK_ID.as_str().to_string();

        let mut conn = self.pool.get().map_err(|e| {
            Error::InvalidOperation(format!("Failed to get connection: {}", e))
        })?;

        // Single UPSERT: SQLite's INSERT OR REPLACE inserts if the row
        // doesn't exist or replaces it when the primary key conflicts.
        // This replaces the previous two-statement update-then-insert pattern.
        diesel::replace_into(timers::table)
            .values(&timer_db)
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to save timer: {}", e),
            })?;

        Ok(())
    }
}
