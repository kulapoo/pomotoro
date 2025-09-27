use std::time::Duration;

use domain::{Config, TimerConfiguration, TimerState, TimerStatus};
use usecases::{
    CreateTaskCmd, create_task,
    timer::{StartTimerSessionCmd, complete_timer_phase, start_timer_session},
};

use crate::utils::{setup::setup_ctx, task::get_active_task, timer::get_timer};

#[tokio::test]
async fn timer_should_complete_full_pomodoro_cycle() {
    let ctx = setup_ctx("timer_should_complete_full_pomodoro_cycle").await;

    // AAA

    // Arrange
    // TODO: 1. Configure test with specific Pomodoro settings
    //    - Set work_duration (e.g., 25 minutes or shorter for testing)
    //    - Set short_break_duration (e.g., 5 minutes or shorter for testing)
    //    - Set long_break_duration (e.g., 15 minutes or shorter for testing)
    //    - Set sessions_before_long_break (e.g., 4)

    let default_config = Config {
        timer: TimerConfiguration::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
        )
        .expect("Failed to create timer configuration"),
        ..Default::default()
    };

    // Act
    let task1_result = create_task(
        ctx.task_repo.clone(),
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Task1".to_string(),
            description: None,
            max_sessions: 4,
            tags: Vec::new(),
            config: Some(default_config),
        },
    )
    .await;

    let task1_id = task1_result.clone().expect("Failed to create task").id;

    let session0_timer_is_idle = get_timer(&ctx).await.state().is_idle();

    // TODO: 2. Start the timer
    //    - Call timer.start() or equivalent
    //    - Assert timer is in Working state
    //    - Assert current session is 1

    let timer_session1_result = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task1_id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session1_timer_is_work_phase_before_completion =
        get_timer(&ctx).await.state().is_work_phase();

    let session1_timer_status = get_timer(&ctx).await.state().status();

    let task_1_current_sessions = get_active_task(&ctx).await.current_sessions;

    // TODO: 3. Complete first work session
    //    - Mock/advance time by work_duration
    //    - Verify timer transitions to ShortBreak state
    //    - Assert session count remains at 1 (or increments based on logic)

    let session1_result = complete_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let session1_timer_is_break_phase =
        get_timer(&ctx).await.state().is_break_phase();

    let task_2_current_sessions = get_active_task(&ctx).await.current_sessions;

    // TODO: 4. Complete first short break
    //    - Mock/advance time by short_break_duration
    //    - Verify timer transitions back to Working state
    //    - Assert now on session 2

    // TODO: 5. Repeat work/short break cycle for remaining sessions
    //    - Complete session 2 -> short break -> session 3
    //    - Complete session 3 -> short break -> session 4

    // TODO: 6. Complete final work session before long break
    //    - Mock/advance time by work_duration for session 4
    //    - Verify timer transitions to LongBreak state (not ShortBreak)
    //    - Assert this triggers after sessions_before_long_break count

    // TODO: 7. Complete long break
    //    - Mock/advance time by long_break_duration
    //    - Verify timer resets cycle (back to session 1 or Idle)
    //    - Assert session counter resets to 0 or 1

    // TODO: 8. Optionally verify second cycle starts correctly
    //    - Start timer again
    //    - Verify it begins at session 1 in Working state

    // TODO: 9. Clean up test resources
    //    - Stop timer if needed
    //    - Clean up any test database or state

    // Assert

    assert_eq!(task1_result.is_ok(), true);
    assert_eq!(session0_timer_is_idle, true);
    assert_eq!(timer_session1_result.is_ok(), true);
    assert_eq!(session1_timer_is_work_phase_before_completion, true);
    assert_eq!(session1_timer_status, TimerStatus::Running);
    assert_eq!(task_1_current_sessions, 0); // Session not incremented when starting, only when completing
    assert_eq!(session1_result.is_ok(), true);
    assert_eq!(session1_timer_is_break_phase, true);
    assert_eq!(task_2_current_sessions, 1); // First work session completed, so now 1
}
