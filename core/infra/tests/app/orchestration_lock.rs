//! Tests for the backend re-entry guard that serializes mutating timer
//! orchestrations (Tauri commands, tray handlers, CountdownExpiredHandler).
//!
//! Without serialization, two overlapping orchestrations race on the shared
//! `TimerTickService` and the singleton timer row. On fast hardware the
//! window is microseconds wide and rarely hit; on macOS M1 it is reliable.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crate::core::context::AppContext;

/// 10 concurrent callers of `orchestration_lock` must run strictly one at a
/// time. The counter increments on entry, decrements on exit, and tracks the
/// max value seen — which must be 1.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn orchestration_lock_serializes_concurrent_callers() {
    let ctx = AppContext::with_name(Some(
        "orchestration_lock_serializes_concurrent_callers",
    ))
    .await
    .expect("Failed to build test context");
    let svc = ctx.timer_tick_service.clone();

    let active = Arc::new(AtomicU32::new(0));
    let max_seen = Arc::new(AtomicU32::new(0));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let svc = svc.clone();
        let active = active.clone();
        let max_seen = max_seen.clone();
        handles.push(tokio::spawn(async move {
            let _guard = svc.orchestration_lock().await;
            let now = active.fetch_add(1, Ordering::SeqCst) + 1;
            // fetch_max via compare-exchange loop
            let mut cur = max_seen.load(Ordering::SeqCst);
            while now > cur {
                match max_seen.compare_exchange(
                    cur,
                    now,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(actual) => cur = actual,
                }
            }
            // Hold the guard for a bit so concurrency would be detectable.
            tokio::time::sleep(Duration::from_millis(20)).await;
            active.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.await.expect("worker panicked");
    }

    assert_eq!(
        max_seen.load(Ordering::SeqCst),
        1,
        "orchestration_lock must serialize callers — saw concurrent execution"
    );
}

/// The guard returned by `orchestration_lock` releases on drop. Once a caller
/// completes, the next waiter proceeds immediately.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestration_lock_releases_on_drop() {
    let ctx =
        AppContext::with_name(Some("orchestration_lock_releases_on_drop"))
            .await
            .expect("Failed to build test context");
    let svc = ctx.timer_tick_service.clone();

    // First caller acquires and releases quickly.
    {
        let _g = svc.orchestration_lock().await;
    }

    // Second caller must acquire within 100ms (no other holder).
    let acquired = tokio::time::timeout(
        Duration::from_millis(100),
        svc.orchestration_lock(),
    )
    .await;
    assert!(
        acquired.is_ok(),
        "orchestration_lock should be acquirable immediately after the previous guard dropped"
    );
}
