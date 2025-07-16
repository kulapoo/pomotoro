use std::time::Duration;
use tokio::time::{sleep, timeout};

pub struct TimeUtils;

impl TimeUtils {
    /// Convert minutes to Duration
    pub fn minutes(minutes: u64) -> Duration {
        Duration::from_secs(minutes * 60)
    }

    /// Convert seconds to Duration
    pub fn seconds(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }

    /// Convert milliseconds to Duration
    pub fn millis(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// Sleep for the specified duration
    pub async fn sleep_for(duration: Duration) {
        sleep(duration).await;
    }

    /// Sleep for the specified number of milliseconds
    pub async fn sleep_millis(millis: u64) {
        sleep(Duration::from_millis(millis)).await;
    }

    /// Sleep for the specified number of seconds
    pub async fn sleep_secs(secs: u64) {
        sleep(Duration::from_secs(secs)).await;
    }

    /// Run an async operation with a timeout
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

/// Macro to create a Duration from minutes
#[macro_export]
macro_rules! minutes {
    ($minutes:expr) => {
        std::time::Duration::from_secs($minutes * 60)
    };
}

/// Macro to create a Duration from seconds
#[macro_export]
macro_rules! seconds {
    ($seconds:expr) => {
        std::time::Duration::from_secs($seconds)
    };
}

/// Macro to create a Duration from milliseconds
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

/// Macro to assert that a duration is approximately equal with default tolerance
#[macro_export]
macro_rules! assert_duration_approx_default {
    ($actual:expr, $expected:expr) => {
        assert_duration_approx!($actual, $expected, std::time::Duration::from_millis(100));
    };
}