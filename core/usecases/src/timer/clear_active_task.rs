use domain::{Result, TimerRepository};
use std::sync::Arc;

/// Detach the active task from the timer, resetting it to `Idle`.
///
/// Used when the active task can no longer run (e.g. it was just completed
/// and there is no successor to advance to) and the timer should not remain
/// bound to it. The UI is expected to prompt the user to select a new task.
///
/// Unlike [`super::reset_timer_to_idle`], which only resets the state
/// machine while keeping a task bound, this fully detaches the task: the
/// timer cannot run again until a new task is attached via `set_task_id`.
///
/// Idempotent: if the timer already has no task attached, this is a no-op
/// and performs no write.
///
/// ## Business Rules
///
/// - No domain event is published here. The triggering use case (e.g.
///   task completion) is responsible for the `TaskCompleted` event, and the
///   app layer is responsible for notifying the UI (mirroring the
///   `AUTO_ADVANCED` emit pattern).
pub async fn clear_active_task(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
) -> Result<()> {
    let mut timer = timer_repo.get().await?;

    if timer.task_id().is_some() {
        timer.clear_task_id();
        timer_repo.save(&timer).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::{TaskId, Timer};
    use std::sync::Mutex;

    /// Minimal in-memory `TimerRepository` for exercising the use case.
    struct FakeTimerRepo {
        timer: Mutex<Timer>,
        save_calls: Mutex<u32>,
    }

    impl FakeTimerRepo {
        fn new(timer: Timer) -> Arc<Self> {
            Arc::new(Self {
                timer: Mutex::new(timer),
                save_calls: Mutex::new(0),
            })
        }

        fn snapshot(&self) -> Timer {
            self.timer.lock().unwrap().clone()
        }

        fn save_calls(&self) -> u32 {
            *self.save_calls.lock().unwrap()
        }
    }

    #[async_trait]
    impl TimerRepository for FakeTimerRepo {
        async fn get(&self) -> std::result::Result<Timer, domain::TimerError> {
            Ok(self.timer.lock().unwrap().clone())
        }

        async fn save(
            &self,
            timer: &Timer,
        ) -> std::result::Result<(), domain::TimerError> {
            let mut slot = self.timer.lock().unwrap();
            *slot = timer.clone();
            *self.save_calls.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[tokio::test]
    async fn detaches_an_active_task() {
        let timer = Timer::new(TaskId::new());
        assert!(timer.task_id().is_some());

        let repo = FakeTimerRepo::new(timer);
        clear_active_task(repo.clone()).await.unwrap();

        assert!(repo.snapshot().task_id().is_none());
        assert!(repo.snapshot().is_idle());
        assert_eq!(repo.save_calls(), 1);
    }

    #[tokio::test]
    async fn is_a_noop_when_already_idle() {
        let repo = FakeTimerRepo::new(Timer::idle());

        clear_active_task(repo.clone()).await.unwrap();

        assert!(repo.snapshot().task_id().is_none());
        // No task was attached, so no write should occur.
        assert_eq!(repo.save_calls(), 0);
    }

    #[tokio::test]
    async fn detaches_even_when_state_machine_is_running() {
        // A completed task may leave the timer mid-phase; clearing must
        // drop the task regardless of the current state machine.
        let timer = Timer::with_state(
            TaskId::new(),
            domain::TimerState::Working {
                remaining_seconds: 42,
            },
        );

        let repo = FakeTimerRepo::new(timer);
        clear_active_task(repo.clone()).await.unwrap();

        let snap = repo.snapshot();
        assert!(snap.task_id().is_none());
        assert!(snap.is_idle());
    }
}
