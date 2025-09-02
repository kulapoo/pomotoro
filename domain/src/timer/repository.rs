use async_trait::async_trait;
use super::{Timer, TimerId, TimerState, Result};

#[async_trait]
pub trait TimerRepository: Send + Sync {
    async fn create(&self, timer: Timer) -> Result<()>;
    async fn get_by_id(&self, id: TimerId) -> Result<Option<Timer>>;
    async fn save(&self, timer: Timer) -> Result<()>;
    async fn delete(&self, id: TimerId) -> Result<()>;
    async fn exists(&self, id: TimerId) -> Result<bool>;
    async fn save_timer_state(&self, timer: &Timer) -> Result<()>;
    async fn load_timer_state(&self) -> Result<Option<TimerState>>;
}