use std::time::Duration;
use tokio::time::{sleep, timeout};

pub struct TimeUtils;

impl TimeUtils {
    pub fn minutes(minutes: u64) -> Duration {
        Duration::from_secs(minutes * 60)
    }

    pub fn seconds(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }

    pub fn millis(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    pub async fn sleep_for(duration: Duration) {
        sleep(duration).await;
    }

    pub async fn sleep_millis(millis: u64) {
        sleep(Duration::from_millis(millis)).await;
    }

    pub async fn sleep_secs(secs: u64) {
        sleep(Duration::from_secs(secs)).await;
    }

    pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, tokio::time::error::Elapsed>
    where
        F: std::future::Future<Output = T>,
    {
        timeout(duration, future).await
    }

    /// Wait for a condition to be true with polling
    pub async fn wait_for_condition<F>(
        condition: F,
        timeout_duration: Duration,
        poll_interval: Duration,
    ) -> Result<(), String>
    where
        F: Fn() -> bool,
    {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout_duration {
            if condition() {
                return Ok(());
            }
            sleep(poll_interval).await;
        }
        
        Err("Timeout waiting for condition".to_string())
    }

    /// Wait for an async condition to be true with polling
    pub async fn wait_for_async_condition<F, Fut>(
        condition: F,
        timeout_duration: Duration,
        poll_interval: Duration,
    ) -> Result<(), String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout_duration {
            if condition().await {
                return Ok(());
            }
            sleep(poll_interval).await;
        }
        
        Err("Timeout waiting for async condition".to_string())
    }
}

#[macro_export]
macro_rules! minutes {
    ($minutes:expr) => {
        std::time::Duration::from_secs($minutes * 60)
    };
}

#[macro_export]
macro_rules! seconds {
    ($seconds:expr) => {
        std::time::Duration::from_secs($seconds)
    };
}

#[macro_export]
macro_rules! millis {
    ($millis:expr) => {
        std::time::Duration::from_millis($millis)
    };
}

/// Macro to assert that a duration is approximately equal to another duration
#[macro_export]
macro_rules! assert_duration_approx {
    ($actual:expr, $expected:expr, $tolerance:expr) => {
        let actual = $actual;
        let expected = $expected;
        let tolerance = $tolerance;
        
        let diff = if actual > expected {
            actual - expected
        } else {
            expected - actual
        };
        
        assert!(
            diff <= tolerance,
            "Duration assertion failed: actual={:?}, expected={:?}, tolerance={:?}, diff={:?}",
            actual, expected, tolerance, diff
        );
    };
}

#[macro_export]
macro_rules! assert_duration_approx_default {
    ($actual:expr, $expected:expr) => {
        assert_duration_approx!($actual, $expected, std::time::Duration::from_millis(100));
    };
}