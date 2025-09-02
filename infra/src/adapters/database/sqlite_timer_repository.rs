use async_trait::async_trait;
use diesel::prelude::*;
use domain::{Timer, TimerId, TimerRepository, TimerState};
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
    async fn create(&self, timer: Timer) -> Result<()> {
        let timer_db = TimerDb::from(timer);
        let mut conn = self.pool.get().map_err(|e| Error::InvalidOperation(format!("Failed to get connection: {}", e)))?;
        
        diesel::insert_into(timers::table)
            .values(&timer_db)
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to create timer: {}", e)
            })?;
        
        Ok(())
    }
    
    async fn get_by_id(&self, id: TimerId) -> Result<Option<Timer>> {
        let mut conn = self.pool.get().map_err(|e| Error::InvalidOperation(format!("Failed to get connection: {}", e)))?;
        
        let timer_db = timers::table
            .filter(timers::id.eq(id.to_string()))
            .first::<TimerDb>(&mut conn)
            .optional()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get timer by id: {}", e)
            })?;
        
        match timer_db {
            Some(db) => {
                Timer::try_from(db)
                    .map(Some)
                    .map_err(|e| Error::RepositoryError {
                        message: format!("Failed to convert timer from database: {}", e)
                    })
            },
            None => Ok(None),
        }
    }
    
    async fn save(&self, timer: Timer) -> Result<()> {
        let timer_db = TimerDb::from(timer.clone());
        let mut conn = self.pool.get().map_err(|e| Error::InvalidOperation(format!("Failed to get connection: {}", e)))?;
        
        diesel::update(timers::table.filter(timers::id.eq(timer.id().to_string())))
            .set(&timer_db)
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to update timer: {}", e)
            })?;
        
        Ok(())
    }
    
    async fn delete(&self, id: TimerId) -> Result<()> {
        let mut conn = self.pool.get().map_err(|e| Error::InvalidOperation(format!("Failed to get connection: {}", e)))?;
        
        diesel::delete(timers::table.filter(timers::id.eq(id.to_string())))
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to delete timer: {}", e)
            })?;
        
        Ok(())
    }
    
    async fn exists(&self, id: TimerId) -> Result<bool> {
        let mut conn = self.pool.get().map_err(|e| Error::InvalidOperation(format!("Failed to get connection: {}", e)))?;
        
        let exists = timers::table
            .filter(timers::id.eq(id.to_string()))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to check timer existence: {}", e)
            })?;
        
        Ok(exists > 0)
    }
    
    async fn save_timer_state(&self, timer: &Timer) -> Result<()> {
        // For now, just save the entire timer
        self.save(timer.clone()).await
    }
    
    async fn load_timer_state(&self) -> Result<Option<TimerState>> {
        // For now, return None as we don't have persistent timer state
        // In the future, this could load from a separate timer_state table
        Ok(None)
    }
}