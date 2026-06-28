use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::adapters::events::mem_event_bus::EventPublisherArc;
use domain::TimerRepository;
use domain::{
    ConfigRepository, Error, Phase, Result as DomainResult, TaskId, Timer,
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
    ///
    /// The shared helper `complete_task_flow` likewise assumes its caller
    /// holds this lock — both of its callers (`complete_task` command,
    /// `menu_complete` tray) acquire it. Do not lock inside it.
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
    ) -> Result<(), String> {
        // Reload timer from repository to ensure we have the latest state
        // This is crucial because the use case just saved the timer
        self.load_state()
            .await
            .map_err(|e| format!("Failed to reload timer state: {}", e))?;

        let config = self.resolve_timer_config(timer_config).await?;
        // Hold the cancel_handle lock for the entire abort→spawn→store sequence so
        // that concurrent calls are serialized. Without this, two callers could both
        // observe an empty handle, spawn separate loops, and orphan one of them.
        let mut cancel_guard = self.abort_existing_loop().await;

        let handle = tokio::spawn(run_tick_loop(
            Arc::clone(&self.timer),
            Arc::clone(&self.event_publisher),
            config,
        ));

        // Store the new handle and release the lock.
        *cancel_guard = Some(handle);
        drop(cancel_guard);

        Ok(())
    }

    /// Stop the timer tick loop
    pub async fn stop_timer_tick_loop(&self) -> DomainResult<()> {
        let _guard = self.abort_existing_loop().await;
        Ok(())
    }

    /// Abort any running tick-loop task and return the `cancel_handle` guard
    /// so the caller can hold it across the subsequent spawn+store sequence
    /// (serialization contract documented on `start_timer_tick_loop`).
    async fn abort_existing_loop(
        &self,
    ) -> tokio::sync::MutexGuard<'_, Option<tokio::task::JoinHandle<()>>> {
        let mut guard = self.cancel_handle.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
        guard
    }

    /// Resolve the timer config from the caller's explicit value, falling
    /// back to the persisted config repository default.
    async fn resolve_timer_config(
        &self,
        timer_config: Option<TimerConfiguration>,
    ) -> Result<TimerConfiguration, String> {
        if let Some(config) = timer_config {
            Ok(config)
        } else {
            self.config_repository
                .get_config()
                .await
                .map_err(|e| e.to_string())
                .map(|c| c.timer)
        }
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

struct TickOutcome {
    should_continue: bool,
    phase_completed: bool,
    events_to_publish: Vec<Box<dyn domain::Event>>,
    expiry_payload: Option<(Phase, TaskId)>,
}

/// Compute one tick of the timer within a single critical section.
///
/// Collapses the previous double-lock (tick computation + post-phase
/// payload read) into one lock scope, capturing the `CountdownExpired`
/// payload (`phase`, `task_id`) up front when the phase completes.
async fn compute_tick_outcome(
    timer: &tokio::sync::Mutex<Timer>,
    config: &TimerConfiguration,
) -> TickOutcome {
    let mut timer = timer.lock().await;
    if !timer.is_running() {
        return TickOutcome {
            should_continue: false,
            phase_completed: false,
            events_to_publish: Vec::new(),
            expiry_payload: None,
        };
    }
    let Some(active) = timer.as_active_mut() else {
        return TickOutcome {
            should_continue: false,
            phase_completed: false,
            events_to_publish: Vec::new(),
            expiry_payload: None,
        };
    };
    match active.tick(config) {
        Ok((phase_complete, events)) => {
            let expiry_payload = if phase_complete {
                timer.task_id().map(|tid| (timer.get_current_phase(), tid))
            } else {
                None
            };
            TickOutcome {
                should_continue: !phase_complete,
                phase_completed: phase_complete,
                events_to_publish: events,
                expiry_payload,
            }
        }
        Err(e) => {
            log::error!("Timer tick error: {e}");
            TickOutcome {
                should_continue: false,
                phase_completed: false,
                events_to_publish: Vec::new(),
                expiry_payload: None,
            }
        }
    }
}

/// Body of the spawned tick loop. Ticks every second, publishes domain
/// events outside the timer lock, and emits a single `CountdownExpired`
/// when a phase completes naturally before exiting.
async fn run_tick_loop(
    timer: Arc<tokio::sync::Mutex<Timer>>,
    event_publisher: EventPublisherArc,
    config: TimerConfiguration,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.tick().await;
    loop {
        interval.tick().await;
        let outcome = compute_tick_outcome(&timer, &config).await;
        if !outcome.events_to_publish.is_empty() {
            event_publisher.publish_batch(outcome.events_to_publish);
        }
        if outcome.phase_completed {
            if let Some((current_phase, task_id)) = outcome.expiry_payload {
                let expiration_event =
                    domain::timer::events::CountdownExpired::new(
                        current_phase,
                        task_id,
                    );
                event_publisher.publish(Box::new(expiration_event));
            }
            break;
        }
        if !outcome.should_continue {
            break;
        }
    }
}
