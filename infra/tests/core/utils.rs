use std::any::TypeId;

use domain::{
    Task, TaskId, TaskRepository, Timer, TimerRepository, TimerState,
};

use crate::{AppContext, AppContextBuilder, UiSimulator};

pub mod assert_utils {
    use super::*;
    pub fn assert_event_published(ctx: &AppContext, event_type: TypeId) {
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
}

pub mod task {
    use super::*;
    pub async fn get_active_task(ctx: &AppContext) -> Task {
        let timer = timer::get_timer(ctx).await;

        let active_task_id = timer.active_task_id().unwrap();

        let task = if let Some(task) = ctx
            .task_repo
            .get_by_id(active_task_id)
            .await
            .expect("Failed to get active task")
        {
            task
        } else {
            ctx.task_repo
                .get_default_task()
                .await
                .unwrap()
                .expect("Failed to get default task")
        };

        task
    }

    pub async fn switch_task(ctx: &AppContext, task_id: TaskId) -> () {
        let mut timer = ctx.timer_repo.get().await.unwrap();
        timer.set_active_task(task_id);
        ctx.timer_repo.save(&timer).await.unwrap()
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
