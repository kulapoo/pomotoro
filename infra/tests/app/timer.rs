use crate::AppContextBuilder;
use domain::{Phase, TimerStatus, event_names, timer::TimerService};
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