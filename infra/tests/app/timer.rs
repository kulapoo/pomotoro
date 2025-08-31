use crate::{AppContextBuilder, UiSimulatorBuilder};
use domain::{Phase, timer::TimerService};
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

    let _ = (*ctx.ui_simulator).clone().start_auto_responder();

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

}
