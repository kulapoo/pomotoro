use std::any::TypeId;

use domain::{Task, TaskRepository, Timer, TimerRepository, TimerState};

use crate::{AppContext, AppContextBuilder, UiSimulator};

pub mod assert_utils {
    use super::*;
    pub fn assert_event_subscribed(ctx: &AppContext, event_type: TypeId) {
        assert!(ctx.event_bus.has_event_type(event_type));
    }

    pub fn assert_event_was_emitted(
        ui_simulator: &UiSimulator,
        event_type: &str,
    ) {
        let events = ui_simulator.app_handle().emitted_events();
        assert!(
            ui_simulator.app_handle().was_event_emitted(event_type),
            "Expected timer:status_changed event to be emitted, but got: {:?}",
            events.iter().map(|e| &e.event_name).collect::<Vec<_>>()
        );
    }
}

pub mod setup {
    use crate::TestDatabase;

    use super::*;
    pub async fn setup_ctx(name: &str) -> AppContext {
        let builder = AppContextBuilder::new()
            .with_name(name)
            .with_standard_fixtures();

        builder.build().await.expect("Failed to build test context")
    }

    pub async fn setup_ctx_with_timer(name: &str) -> AppContext {
        let builder = AppContextBuilder::new()
            .with_name(name)
            .with_timer_started()
            .with_standard_fixtures();

        builder.build().await.expect("Failed to build test context")
    }

    pub async fn setup_ctx_with_existing_db(db: TestDatabase) -> AppContext {
        let builder = AppContextBuilder::new()
            .with_existing_db(db)
            .with_standard_fixtures();

        builder.build().await.expect("Failed to build test context")
    }
}

pub mod task {
    use super::*;
    pub async fn get_active_task(ctx: &AppContext) -> Task {
        let timer = timer::get_timer(ctx).await;

        let active_task_id = match timer.task_id() {
            Some(id) => id,
            None => {
                // No task attached — fall back to any task in the repo.
                return ctx
                    .task_repo
                    .get_all()
                    .await
                    .expect("Failed to load tasks")
                    .into_iter()
                    .next()
                    .expect("No tasks in repo");
            }
        };

        ctx.task_repo
            .get_by_id(active_task_id)
            .await
            .expect("Failed to get active task")
            .expect("Active task not found in repo")
    }
}

pub mod timer {
    use super::*;
    pub async fn get_timer_state(ctx: &AppContext) -> TimerState {
        let timer = ctx.timer_repo.get().await.unwrap();
        timer.state().clone()
    }

    pub async fn get_timer(ctx: &AppContext) -> Timer {
        ctx.timer_repo.get().await.unwrap()
    }

    pub async fn load_timer_state(ctx: &AppContext) -> () {
        // Load state is now handled directly through the repository
        let _ = ctx.timer_repo.get().await.unwrap();
    }
}
