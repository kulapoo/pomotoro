use crate::AppContextBuilder;
use domain::{Phase, timer::TimerService, event_names};
use usecases::timer::{start_timer_session, StartTimerSessionCmd};

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
        .build()
        .await
        .expect("Failed to build context");

    let simulator = (*ctx.ui_simulator).clone().start_listen_to_events();

    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");

    let start_timer_session_cmd = StartTimerSessionCmd {
        task_id: timer_state.active_entity_id()
    };

    start_timer_session(
        ctx.timer_service.clone(),
        ctx.task_repo.clone(),
        ctx.event_bus.clone(),
        start_timer_session_cmd
    ).await.expect("Failed to start timer session");

    // Wait a bit for async event handlers to execute
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check what events were actually emitted
    let events = simulator.app_handle().emitted_events();

    assert!(
        simulator.app_handle().was_event_emitted(event_names::ui_listeners::timer::STATUS_CHANGED),
        "Expected timer:status_changed event to be emitted, but got: {:?}",
        events.iter().map(|e| &e.event_name).collect::<Vec<_>>()
    );
}
