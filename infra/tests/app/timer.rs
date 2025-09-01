use std::any::TypeId;

use crate::AppContextBuilder;
use domain::{event_names, timer::TimerService, Phase, TimerStarted, TimerStatus};
use usecases::timer::{
    pause_timer_session, reset_timer_session, start_timer_session, StartTimerSessionCmd
};

#[tokio::test]
async fn timer_should_initialize_in_idle_state() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");
    let timer = ctx
        .timer_service
        .get_timer()
        .await
        .expect("Failed to get timer");

    assert!(timer_state.is_idle());
    assert_eq!(timer.get_current_phase(), Phase::Work);
    assert_eq!(timer_state.session_count(), 0);
    assert_eq!(timer_state.remaining_seconds(), 25 * 60);
}

#[tokio::test]
async fn timer_should_start_from_idle_state() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .with_timer_started()
        .build()
        .await
        .expect("Failed to build context");
    let event_bus = ctx.event_bus.clone();

    let simulator = (*ctx.ui_simulator).clone().start_listen_to_events();

    // Check what events were actually emitted
    let events = simulator.app_handle().emitted_events();

    assert!(
        simulator.app_handle().was_event_emitted(
            event_names::ui_listeners::timer::STATUS_CHANGED
        ),
        "Expected timer:status_changed event to be emitted, but got: {:?}",
        events.iter().map(|e| &e.event_name).collect::<Vec<_>>()
    );

    assert!(event_bus.has_event_type(TypeId::of::<TimerStarted>()));
}

#[tokio::test]
async fn timer_should_not_start_when_already_running() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .with_timer_started()
        .build()
        .await
        .expect("Failed to build context");

    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let start_with_task_cmd = StartTimerSessionCmd {
        task_id: timer_state.active_entity_id(),
    };

    let result_with_task = start_timer_session(
        ctx.timer_service.clone(),
        ctx.task_repo.clone(),
        ctx.event_bus.clone(),
        start_with_task_cmd,
    )
    .await;

    // Should fail with ConfigurationError when trying to switch tasks
    assert!(result_with_task.is_err());
    match result_with_task {
        Err(domain::Error::ConfigurationError { message }) => {
            assert!(
                message.contains("Cannot switch tasks while timer is running")
            );
        }
        _ => panic!(
            "Expected ConfigurationError when switching tasks, got {:?}",
            result_with_task
        ),
    }
}

#[tokio::test]
async fn timer_should_pause_when_running() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .with_timer_started()
        .build()
        .await
        .expect("Failed to build context");

    pause_timer_session(ctx.timer_service.clone(), ctx.event_bus.clone())
        .await
        .expect("Failed to pause timer session");

    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");

    let simulator = (*ctx.ui_simulator).clone().start_listen_to_events();

    // Check what events were actually emitted
    let events = simulator.app_handle().emitted_events();

    assert!(
        simulator.app_handle().was_event_emitted(
            event_names::ui_listeners::timer::STATUS_CHANGED
        ),
        "Expected timer:status_changed event to be emitted, but got: {:?}",
        events.iter().map(|e| &e.event_name).collect::<Vec<_>>()
    );

    assert_eq!(timer_state.status(), TimerStatus::Paused);

    assert!(timer_state.remaining_seconds() < 25 * 60);
}


#[tokio::test]
async fn timer_should_reset_to_initial_state() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .with_timer_started()
        .build()
        .await
        .expect("Failed to build context");

    reset_timer_session(ctx.timer_service.clone(), ctx.task_repo.clone(), ctx.event_bus.clone())
        .await
        .expect("Failed to reset timer session");

    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");

    assert!(timer_state.is_idle());
    assert_eq!(timer_state.session_count(), 0);
    assert_eq!(timer_state.remaining_seconds(), 25 * 60);
}

#[tokio::test]
async fn timer_state_should_persist_across_restarts() {
    use std::sync::Arc;

    let (test_db, status_before, remaining_before, phase_before, session_count_before) = {
        let first_ctx = AppContextBuilder::new()
            .with_name("persistence_test")
            .with_standard_fixtures()
            .with_timer_started()
            .build()
            .await
            .expect("Failed to build first context");

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let state = first_ctx
            .timer_service
            .get_state()
            .await
            .expect("Failed to get timer state before restart");

        let timer = first_ctx
            .timer_service
            .get_timer()
            .await
            .expect("Failed to get timer before restart");

        let status = state.status();
        let remaining = state.remaining_seconds();
        let phase = timer.get_current_phase();
        let session_count = state.session_count();

        (first_ctx.db, status, remaining, phase, session_count)
    };

    let new_pool = test_db.reconnect()
        .expect("Failed to reconnect to database");

    use infra::adapters::{
        database::{SqliteConfigRepository, SqliteTimerRepository},
        events::mem_event_bus::InMemoryEventBus,
        timer::SqliteTimerService
    };

    let event_bus = Arc::new(InMemoryEventBus::new());
    let config_repo = Arc::new(SqliteConfigRepository::new(new_pool.clone()));
    let timer_repo = Arc::new(SqliteTimerRepository::new(new_pool.clone()));
    let timer_service = Arc::new(SqliteTimerService::new(
        event_bus.clone(),
        timer_repo.clone(),
        config_repo.clone(),
    ));

    let state_after = timer_service
        .get_state()
        .await
        .expect("Failed to get timer state after restart");

    let timer_after = timer_service
        .get_timer()
        .await
        .expect("Failed to get timer after restart");

    assert_eq!(state_after.status(), status_before, "Timer status should persist");
    assert_eq!(timer_after.get_current_phase(), phase_before, "Timer phase should persist");
    assert_eq!(state_after.session_count(), session_count_before, "Session count should persist");

    // Remaining time should be approximately the same (allowing small drift for processing time)
    let remaining_after = state_after.remaining_seconds();
    let time_diff = (remaining_before as i32 - remaining_after as i32).abs();
    assert!(
        time_diff <= 2,
        "Remaining time should be preserved (before: {}, after: {}, diff: {})",
        remaining_before,
        remaining_after,
        time_diff
    );
}