//! Regression test for the macOS-M1 freeze root cause.
//!
//! `TimerTickService::save_state` used to hold the `timer` tokio Mutex across
//! the repository write `.await`. On slow disk (M1) this blocked the tick-loop
//! task and any concurrent orchestration that needed the timer, producing an
//! intermittent but reliable freeze. The fix is to clone the timer under the
//! lock, drop the guard, then write.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use domain::timer::Result as TimerResult;
use domain::{ConfigRepository, Timer, TimerRepository};
use infra::adapters::events::mem_event_bus::EventPublisherArc;
use infra::adapters::timer::TimerTickService;
use tokio::sync::Notify;

/// Fake repository whose `save` blocks on a `Notify` until released by the
/// test. Lets us observe whether the timer mutex is held *during* the write.
struct BlockingSaveRepo {
    entered: Arc<Notify>,
    release: Arc<Notify>,
}

#[async_trait]
impl TimerRepository for BlockingSaveRepo {
    async fn get(&self) -> TimerResult<Timer> {
        Ok(Timer::idle())
    }

    async fn save(&self, _timer: &Timer) -> TimerResult<()> {
        self.entered.notify_one();
        self.release.notified().await;
        Ok(())
    }
}

/// Minimal config repo stub — the test never calls it, but `TimerTickService::new`
/// requires it.
struct StubConfigRepo;

#[async_trait]
impl ConfigRepository for StubConfigRepo {
    async fn get_config(&self) -> domain::Result<domain::Config> {
        Ok(domain::Config::default())
    }
    async fn save_config(
        &self,
        _config: &domain::Config,
    ) -> domain::Result<()> {
        Ok(())
    }
    async fn reset_to_defaults(&self) -> domain::Result<domain::Config> {
        Ok(domain::Config::default())
    }
    async fn config_exists(&self) -> domain::Result<bool> {
        Ok(true)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn save_state_does_not_hold_timer_mutex_during_write() {
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());

    let timer_repo: Arc<dyn TimerRepository + Send + Sync> =
        Arc::new(BlockingSaveRepo {
            entered: entered.clone(),
            release: release.clone(),
        });
    let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
        Arc::new(StubConfigRepo);
    let event_publisher: EventPublisherArc = Arc::new(
        infra::adapters::events::mem_event_bus::InMemoryEventBus::new(),
    );

    let svc = Arc::new(TimerTickService::new(
        event_publisher,
        timer_repo,
        config_repo,
    ));

    // Start save_state in the background. It will enter the repo's `save`,
    // notify us, then block on `release`.
    let svc_clone = Arc::clone(&svc);
    let save_task = tokio::spawn(async move { svc_clone.save_state().await });

    // Wait until `save` is in flight.
    entered.notified().await;

    // While `save_state` is still awaiting the repo, try to acquire the timer
    // mutex via `with_timer`. With Fix C applied, this succeeds immediately.
    // Without Fix C (mutex held across the write), this times out.
    let probe = tokio::time::timeout(
        Duration::from_millis(200),
        svc.with_timer(|_t| ()),
    )
    .await;

    assert!(
        probe.is_ok(),
        "timer mutex must be releasable while save_state awaits the repository \
         write. Holding it across the `.await` is the macOS-M1 freeze root cause."
    );

    // Let the background save complete so the test can tear down cleanly.
    release.notify_one();
    save_task.await.unwrap().expect("save_state returned err");
}
