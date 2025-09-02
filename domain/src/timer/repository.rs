use async_trait::async_trait;
use super::{Timer, Result};

#[async_trait]
pub trait TimerRepository: Send + Sync {
    /// Get the single timer instance
    async fn get(&self) -> Result<Timer>;
    
    /// Save the timer state
    async fn save(&self, timer: &Timer) -> Result<()>;
}