use std::any::TypeId;

use crate::AppContextBuilder;
use domain::{
    Phase, TaskRepository, TaskStatus, TimerRepository, TimerStarted, 
    timer::{Status as TimerStatus, TimerService}, event_names,
};
use usecases::{CreateTaskCmd, create_task};
use usecases::timer::{
    StartTimerSessionCmd, pause_timer_session, reset_timer_session,
    start_timer_session,
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
        .build()
        .await
        .expect("Failed to build context");
    
    // Create a task
    let task = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Start Test Task".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create task");

    // Verify timer is idle before starting
    let timer_state_before = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");
    
    assert!(timer_state_before.is_idle(), "Timer should be idle initially");

    let event_bus = ctx.event_bus.clone();
    let simulator = (*ctx.ui_simulator).clone().start_listen_to_events();

    // Start timer with the task
    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task.id.to_string()),
        },
    )
    .await
    .expect("Failed to start timer");

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
    
    // Get the task's timer directly from the repository
    let task_from_repo = ctx.task_repo
        .get_by_id(task.id)
        .await
        .expect("Failed to get task")
        .expect("Task should exist");
    
    let timer = ctx.timer_repo
        .get_by_id(task_from_repo.get_timer_id())
        .await
        .expect("Failed to get timer")
        .expect("Timer should exist");
    
    assert!(timer.is_running(), "Timer should be running");
    assert_eq!(timer.state().status(), TimerStatus::Running);
    assert_eq!(
        timer.state().active_entity_id(),
        Some(task.id.to_string()),
        "Timer should be associated with the task"
    );
}

#[tokio::test]
async fn timer_should_not_start_when_already_running() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    // Create a task and start timer with it
    let task = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Running Task".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create task");

    // Start timer with the task
    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task.id.to_string()),
        },
    )
    .await
    .expect("Failed to start timer");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to start timer again with the same task - should fail
    let result_same_task = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task.id.to_string()),
        },
    )
    .await;

    // Should fail with InvalidStateTransition
    assert!(result_same_task.is_err());
    match result_same_task {
        Err(domain::Error::InvalidStateTransition { from, to }) => {
            assert_eq!(to, "Start", "Should not be able to start when already running");
        }
        _ => panic!(
            "Expected InvalidStateTransition error, got {:?}",
            result_same_task
        ),
    }
}

#[tokio::test]
async fn should_prevent_task_switch_while_timer_is_running() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    // Create first task
    let task1 = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Task 1".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create first task");

    // Create second task  
    let task2 = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Task 2".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create second task");

    // Start timer with first task
    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task1.id.to_string()),
        },
    )
    .await
    .expect("Failed to start timer with first task");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to switch to second task while timer is running - should fail
    let switch_result = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task2.id.to_string()),
        },
    )
    .await;

    // Should fail because we can't switch tasks while timer is running
    assert!(switch_result.is_err());
    
    // The actual error type will depend on implementation
    // Could be InvalidStateTransition if trying to start an already running timer
    // Or a specific error for task switching
    match switch_result {
        Err(err) => {
            // Just verify it's an error - exact type depends on implementation
            assert!(
                matches!(err, domain::Error::InvalidStateTransition { .. })
                || matches!(err, domain::Error::ConfigurationError { .. }),
                "Expected error when switching tasks while timer is running, got: {:?}",
                err
            );
        }
        Ok(_) => panic!("Should not be able to switch tasks while timer is running"),
    }
}

#[tokio::test]
async fn timer_should_pause_when_running() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    // Create a task to work with
    let task = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Task to Pause".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create task");

    // Start timer with the task
    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task.id.to_string()),
        },
    )
    .await
    .expect("Failed to start timer");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    pause_timer_session(
        task.id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone()
    )
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
    
    // Verify task association is maintained during pause
    assert_eq!(
        timer_state.active_entity_id(),
        Some(task.id.to_string()),
        "Task association should be maintained during pause"
    );
}

#[tokio::test]
async fn timer_should_reset_to_initial_state() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    // Create a task
    let task = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Task to Reset".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create task");

    // Start timer with the task
    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task.id.to_string()),
        },
    )
    .await
    .expect("Failed to start timer");

    // Let some time pass
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify timer is running with the task
    let timer_state_before = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state before reset");
    
    assert_eq!(timer_state_before.status(), TimerStatus::Running);
    assert_eq!(
        timer_state_before.active_entity_id(),
        Some(task.id.to_string())
    );
    
    // Reset the timer
    reset_timer_session(
        task.id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("Failed to reset timer session");

    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state after reset");

    assert!(timer_state.is_idle());
    assert_eq!(timer_state.session_count(), 0);
    assert_eq!(timer_state.remaining_seconds(), 25 * 60);
    
    // After reset, the task association should be cleared or maintained depending on implementation
    // The timer is idle so it may or may not maintain the last task association
}

#[tokio::test]
async fn timer_state_should_persist_across_restarts() {
    use std::sync::Arc;

    let (
        test_db,
        status_before,
        remaining_before,
        phase_before,
        session_count_before,
        task_id,
        active_entity_before,
    ) = {
        let first_ctx = AppContextBuilder::new()
            .with_name("persistence_test")
            .with_standard_fixtures()
            .build()
            .await
            .expect("Failed to build first context");

        // Create a task to associate with the timer
        let task = create_task(
            first_ctx.task_repo.clone(),
            first_ctx.timer_repo.clone(),
            first_ctx.event_bus.clone(),
            CreateTaskCmd {
                name: "Persistent Task".to_string(),
                description: Some("Task for persistence test".to_string()),
                max_sessions: 4,
                tags: vec!["persistence".to_string()],
            },
        )
        .await
        .expect("Failed to create task");

        // Start timer with the task
        start_timer_session(
            first_ctx.task_repo.clone(),
            first_ctx.timer_repo.clone(),
            first_ctx.event_bus.clone(),
            StartTimerSessionCmd {
                task_id: Some(task.id.to_string()),
            },
        )
        .await
        .expect("Failed to start timer");

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
        let active_entity = state.active_entity_id();

        (first_ctx.db, status, remaining, phase, session_count, task.id, active_entity)
    };

    let new_pool = test_db
        .reconnect()
        .expect("Failed to reconnect to database");

    use infra::adapters::{
        database::{SqliteConfigRepository, SqliteTimerRepository, SqliteTaskRepository},
        events::mem_event_bus::InMemoryEventBus,
        timer::SqliteTimerService,
    };

    let event_bus = Arc::new(InMemoryEventBus::new());
    let config_repo = Arc::new(SqliteConfigRepository::new(new_pool.clone()));
    let timer_repo = Arc::new(SqliteTimerRepository::new(new_pool.clone()));
    let task_repo = Arc::new(SqliteTaskRepository::new(new_pool.clone()));
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

    // Verify task still exists after restart
    let task_after = task_repo
        .get_by_id(task_id)
        .await
        .expect("Failed to get task after restart")
        .expect("Task should exist after restart");

    assert_eq!(
        state_after.status(),
        status_before,
        "Timer status should persist"
    );
    assert_eq!(
        timer_after.get_current_phase(),
        phase_before,
        "Timer phase should persist"
    );
    assert_eq!(
        state_after.session_count(),
        session_count_before,
        "Session count should persist"
    );
    
    // Verify task-timer association persists
    assert_eq!(
        state_after.active_entity_id(),
        active_entity_before,
        "Active task association should persist"
    );
    assert_eq!(
        state_after.active_entity_id(),
        Some(task_id.to_string()),
        "Timer should still be associated with the same task"
    );

    // Verify task properties persist
    assert_eq!(task_after.name, "Persistent Task");
    assert_eq!(task_after.description, Some("Task for persistence test".to_string()));
    assert_eq!(task_after.tags, vec!["persistence".to_string()]);

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

#[tokio::test]
async fn should_start_timer_with_specific_task() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    // Create a specific task for testing
    let created_task = usecases::create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        usecases::CreateTaskCmd {
            name: "Test Task for Timer".to_string(),
            description: Some("Task to test timer association".to_string()),
            max_sessions: 6,
            tags: vec!["test".to_string()],
        },
    )
    .await
    .expect("Failed to create task");

    // Start timer with the specific task
    let start_cmd = StartTimerSessionCmd {
        task_id: Some(created_task.id.to_string()),
    };

    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        start_cmd,
    )
    .await
    .expect("Failed to start timer with task");

    // Verify timer state
    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");

    // Get the timer directly from the repository instead of the service
    let timer = ctx.timer_repo
        .get_by_id(created_task.get_timer_id())
        .await
        .expect("Failed to get timer from repository")
        .expect("Timer should exist");
    
    assert!(timer.is_running(), "Timer should be running");
    assert_eq!(timer.state().status(), TimerStatus::Running);

    // Verify task association
    let task = ctx.task_repo
        .get_by_id(created_task.id)
        .await
        .expect("Failed to get task")
        .expect("Task not found");

    assert_eq!(task.status, TaskStatus::Active);
    
    // Verify events were emitted
    let simulator = (*ctx.ui_simulator).clone().start_listen_to_events();
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
async fn should_switch_active_task_while_timer_is_idle() {
    let ctx = AppContextBuilder::new()
        .with_standard_fixtures()
        .build()
        .await
        .expect("Failed to build context");

    // Create first task
    let task1 = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "First Task".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create first task");

    // Create second task
    let task2 = create_task(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        CreateTaskCmd {
            name: "Second Task".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
        },
    )
    .await
    .expect("Failed to create second task");

    // Start timer with first task
    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task1.id.to_string()),
        },
    )
    .await
    .expect("Failed to start timer with first task");

    // Verify timer is associated with first task
    let timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state");
    
    assert_eq!(
        timer_state.active_entity_id(),
        Some(task1.id.to_string()),
        "Timer should be associated with first task"
    );

    // Reset timer to idle state
    reset_timer_session(
        task1.id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
    )
    .await
    .expect("Failed to reset timer");

    // Verify timer is now idle
    let timer_state_after_reset = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get timer state after reset");
    
    assert!(timer_state_after_reset.is_idle(), "Timer should be idle after reset");

    // Start timer with second task
    start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task2.id.to_string()),
        },
    )
    .await
    .expect("Failed to start timer with second task");

    // Verify timer is now associated with second task
    let final_timer_state = ctx
        .timer_service
        .get_state()
        .await
        .expect("Failed to get final timer state");
    
    assert_eq!(
        final_timer_state.active_entity_id(),
        Some(task2.id.to_string()),
        "Timer should now be associated with second task"
    );
}
