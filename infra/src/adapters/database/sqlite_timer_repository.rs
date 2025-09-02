use async_trait::async_trait;
use diesel::prelude::*;
use domain::{Timer, TimerRepository, DEFAULT_TIMER_ID};
use domain::timer::{Error, Result};
use std::sync::Arc;
use crate::schema::timers;
use super::{DbPool, models::TimerDb};

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
        let mut conn = self.pool.get().map_err(|e| Error::InvalidOperation(format!("Failed to get connection: {}", e)))?;
        let timer_id = DEFAULT_TIMER_ID.to_string();
        
        let timer_db = timers::table
            .filter(timers::id.eq(&timer_id))
            .first::<TimerDb>(&mut conn)
            .optional()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get timer: {}", e)
            })?;
        
        match timer_db {
            Some(db) => {
                Timer::try_from(db)
                    .map_err(|e| Error::RepositoryError {
                        message: format!("Failed to convert timer from database: {}", e)
                    })
            },
            None => {
                // Timer doesn't exist, create the default one
                let timer = Timer::default_timer();
                self.save(&timer).await?;
                Ok(timer)
            }
        }
    }
    
    async fn save(&self, timer: &Timer) -> Result<()> {
        let timer_db = TimerDb::from(timer.clone());
        let mut conn = self.pool.get().map_err(|e| Error::InvalidOperation(format!("Failed to get connection: {}", e)))?;
        let timer_id = DEFAULT_TIMER_ID.to_string();
        
        // Try to update first
        let updated = diesel::update(timers::table.filter(timers::id.eq(&timer_id)))
            .set(&timer_db)
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to update timer: {}", e)
            })?;
        
        // If no rows were updated, insert the timer
        if updated == 0 {
            diesel::insert_into(timers::table)
                .values(&timer_db)
                .execute(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to create timer: {}", e)
                })?;
        }
        
        Ok(())
    }
}