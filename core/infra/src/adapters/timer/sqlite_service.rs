use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

use crate::adapters::events::mem_event_bus::EventPublisherArc;
use domain::TimerRepository;
use domain::{
    ConfigRepository, Error, Result as DomainResult, TaskId, Timer,
    TimerConfiguration,
};

/// Infrastructure service for managing timer tick loops and technical concerns
/// This is NOT a domain service - it handles infrastructure-specific timer management
#[derive(Clone)]
pub struct TimerTickService {
    timer: Arc<Mutex<Timer>>,
    cancel_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Serializes mutating orchestrations (Tauri commands, tray handlers,
    /// `CountdownExpiredHandler`). Held for the entire body of a command.
    /// Without this, two concurrent orchestrations race on the DB and the
    /// tick loop — a race that is invisible on fast hardware but
    /// reproducibly freezes the app on macOS M1.
    ///
    /// Internal methods (`start_timer_tick_loop`, `stop_timer_tick_loop`,
    /// `load_state`, `save_state`) MUST NOT acquire this lock — they assume
    /// the caller holds it. Acquiring it inside those methods would
    /// re-entrantly deadlock.
    orchestration_lock: Arc<tokio::sync::Mutex<()>>,
    event_publisher: EventPublisherArc,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
}

impl TimerTickService {
    pub fn new(
        event_publisher: EventPublisherArc,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    ) -> Self {
        let timer = Timer::idle();

        Self {
            timer: Arc::new(Mutex::new(timer)),
            cancel_handle: Arc::new(Mutex::new(None)),
            orchestration_lock: Arc::new(tokio::sync::Mutex::new(())),
            event_publisher,
            timer_repository,
            config_repository,
        }
    }

    /// Persist the in-memory timer to the repository.
    ///
    /// Acquires `timer` only long enough to clone a snapshot, then drops the
    /// guard BEFORE the repository write. Holding the mutex across the
    /// `.await` wedges the tick-loop task and any concurrent orchestration on
    /// slow disk (macOS M1 reproducibly froze here).
    pub async fn save_state(&self) -> DomainResult<()> {
        let timer_snapshot = {
            let guard = self.timer.lock().await;
            guard.clone()
        };
        self.timer_repository
            .save(&timer_snapshot)
            .await
            .map_err(|e| Error::RepositoryError {
                message: e.to_string(),
            })
    }

    /// Start the infrastructure timer tick loop.
    ///
    /// # Design Contract — Tick-Loop Ownership
    ///
    /// Callers — NOT domain event handlers — own the lifecycle of the tick
    /// loop. The auto-advance race was eliminated by routing start/stop out of
    /// detached event-bus handlers and into the orchestrators that drive the
    /// use cases (Tauri commands, tray handlers, `CountdownExpiredHandler`).
    ///
    /// ## Legitimate callers
    /// 1. App-layer Tauri commands (`apps/tauri-app/src/commands/**`).
    /// 2. App-layer tray handlers (`apps/tauri-app/src/tray.rs`).
    /// 3. Infra event handlers that are the ENTRY POINT of an async flow and
    ///    interpret a usecase outcome (e.g. `CountdownExpiredHandler`).
    ///    Reactors that merely respond to facts MUST NOT call this method.
    /// 4. Test setup.
    ///
    /// ## Sequencing
    /// When an orchestration needs both STOP and START:
    ///     await stop_timer_tick_loop();
    ///     await start_timer_tick_loop(cfg);
    /// Never publish events in lieu of these calls.
    ///
    /// ## Idempotency
    /// `start` aborts any prior handle and overwrites it (last-write-wins).
    /// `stop` is a no-op when no handle is present.
    pub async fn start_timer_tick_loop(
        &self,
        timer_config: Option<TimerConfiguration>,
        _task_id: Option<TaskId>,
    ) -> Result<(), String> {
        // Reload timer from repository to ensure we have the latest state
        // This is crucial because the use case just saved the timer
        self.load_state()
            .await
            .map_err(|e| format!("Failed to reload timer state: {}", e))?;

        // Get configuration from parameter or default from config repository
        let config = if let Some(config) = timer_config {
            config
        } else {
            self.config_repository
                .get_config()
                .await
                .map_err(|e| e.to_string())?
                .timer
        };
        // Hold the cancel_handle lock for the entire abort→spawn→store sequence so
        // that concurrent calls are serialized. Without this, two callers could both
        // observe an empty handle, spawn separate loops, and orphan one of them.
        let mut cancel_guard = self.cancel_handle.lock().await;

        // Cancel any existing timer task while still holding the lock
        if let Some(handle) = cancel_guard.take() {
            handle.abort();
        }

        let timer_clone = Arc::clone(&self.timer);
        let event_publisher_clone = Arc::clone(&self.event_publisher);

        // Move config directly into the spawn — no clone needed.
        // `tokio::spawn` returns immediately, so holding the async mutex guard
        // across this call is safe and introduces no deadlock risk.
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            // Skip the first tick which completes immediately
            interval.tick().await;
            loop {
                interval.tick().await;

                // Hold the lock only for tick computation; collect events to publish outside
                let (should_continue, phase_completed, events_to_publish) = {
                    let mut timer = timer_clone.lock().await;

                    if !timer.is_running() {
                        (false, false, Vec::new())
                    } else {
                        match timer.as_active_mut() {
                            Some(active) => match active.tick(&config) {
                                Ok((phase_complete, events)) => {
                                    (!phase_complete, phase_complete, events)
                                }
                                Err(e) => {
                                    eprintln!("Timer tick error: {e}");
                                    (false, false, Vec::new())
                                }
                            },
                            // A running timer always has a task bound; if
                            // not, stop the loop defensively.
                            None => (false, false, Vec::new()),
                        }
                    }
                };
                // Lock released — publish events outside the critical section
                if !events_to_publish.is_empty() {
                    event_publisher_clone.publish_batch(events_to_publish);
                }

                // If phase completed naturally (countdown reached 0), handle completion
                if phase_completed {
                    // Get the current phase and task_id before breaking.
                    // If the timer has no active task (shouldn't happen
                    // mid-tick, but be defensive), skip the
                    // CountdownExpired event.
                    let maybe_event = {
                        let timer = timer_clone.lock().await;
                        timer
                            .task_id()
                            .map(|tid| (timer.get_current_phase(), tid))
                    };

                    if let Some((current_phase, task_id)) = maybe_event {
                        // Publish the generic CountdownExpired event
                        use domain::timer::events::CountdownExpired;

                        let expiration_event =
                            CountdownExpired::new(current_phase, task_id);

                        event_publisher_clone
                            .publish(Box::new(expiration_event));
                    }

                    break;
                }

                if !should_continue {
                    break;
                }
            }
        });

        // Store the new handle and release the lock.
        *cancel_guard = Some(handle);
        drop(cancel_guard);

        Ok(())
    }

    /// Stop the timer tick loop
    pub async fn stop_timer_tick_loop(&self) -> DomainResult<()> {
        // Cancel the timer task
        {
            let mut cancel_guard = self.cancel_handle.lock().await;
            if let Some(handle) = cancel_guard.take() {
                handle.abort();
            }
        }
        Ok(())
    }

    /// Acquire the global orchestration lock. Hold the returned guard for the
    /// entire body of a mutating entry point (Tauri command, tray handler,
    /// `CountdownExpiredHandler::handle`). When the guard drops, the next
    /// waiter proceeds.
    ///
    /// Internal methods on this service do NOT call this — they assume the
    /// caller has already acquired the lock.
    pub async fn orchestration_lock(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.orchestration_lock.lock().await
    }

    /// Test/observability helper. Returns `true` when a tick-loop task is
    /// currently registered AND still alive (not aborted, not finished).
    ///
    /// Production code MUST NOT branch on this — it exists so regression tests
    /// can assert that an orchestration left the loop in the expected state
    /// without relying on flaky timing.
    ///
    /// Note: deliberately NOT gated on `#[cfg(test)]`. Such gating makes the
    /// helper invisible to integration tests under `tests/` (the library is
    /// not compiled with `cfg(test)` when linked into an integration binary),
    /// which would defeat its purpose.
    pub async fn is_tick_loop_alive(&self) -> bool {
        let guard = self.cancel_handle.lock().await;
        match guard.as_ref() {
            None => false,
            Some(handle) => !handle.is_finished(),
        }
    }

    /// Get the current timer for infrastructure purposes
    pub async fn get_current_timer(&self) -> Timer {
        self.timer.lock().await.clone()
    }

    /// Access the timer by reference without cloning.
    /// Callers that only need to read a field should prefer this over `get_current_timer()`.
    pub async fn with_timer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Timer) -> R,
    {
        let timer = self.timer.lock().await;
        f(&timer)
    }

    /// Update the timer (for infrastructure use only)
    pub async fn update_timer<F>(&self, update_fn: F) -> DomainResult<()>
    where
        F: FnOnce(&mut Timer) -> DomainResult<()>,
    {
        {
            let mut timer = self.timer.lock().await;
            update_fn(&mut timer)?;
        }
        self.save_state().await
    }

    /// Load timer state from repository
    pub async fn load_state(&self) -> DomainResult<()> {
        let loaded_timer = self.timer_repository.get().await.map_err(|e| {
            Error::RepositoryError {
                message: e.to_string(),
            }
        })?;
        {
            *self.timer.lock().await = loaded_timer;
        }
        Ok(())
    }

    /// Reset the timer to initial state
    pub async fn reset_timer(
        &self,
        timer_config: TimerConfiguration,
    ) -> DomainResult<()> {
        // Reset the timer using the domain method (but we won't publish the events)
        {
            let mut timer = self.timer.lock().await;
            // Call reset on the timer - this returns events but we ignore them
            // since the requirement is no event publishing
            let _ = timer.reset(&timer_config)?;
        }

        // Save the reset state to the repository
        self.save_state().await
    }

    pub async fn reset_timer_phase(
        &self,
        timer_config: TimerConfiguration,
    ) -> DomainResult<()> {
        // Reset the timer using the domain method (but we won't publish the events)
        {
            let mut timer = self.timer.lock().await;
            // Call reset on the timer - this returns events but we ignore them
            // since the requirement is no event publishing
            let _ = timer
                .as_active_mut()
                .ok_or(Error::NoActiveTask)?
                .reset_phase(&timer_config)?;
        }

        // Save the reset state to the repository
        self.save_state().await
    }
}
