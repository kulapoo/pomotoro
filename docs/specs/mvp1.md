# MVP Integration Testing Specification - TDD Approach

## Overview
This document defines the integration testing strategy for the Pomotoro MVP using Test-Driven Development (TDD). Each test is designed to be small, focused, and build upon previous tests to create a comprehensive test suite.

## MVP Feature Checklist (Progress Tracking)

Track your implementation progress by checking off completed features:

- [ ] **Timer Core** - Basic start/pause/reset functionality
- [ ] **Timer Phases** - Work/Break phase transitions  
- [ ] **Timer Tick** - Real-time countdown mechanism
- [ ] **Task Creation** - Basic CRUD operations
- [ ] **Task Status** - State transitions (pending/in-progress/completed)
- [ ] **Timer-Task Integration** - Active task during timer sessions
- [ ] **Session Tracking** - Complete work sessions counter
- [ ] **Configuration** - Basic timer and app settings
- [ ] **Task Queue** - Multiple task management
- [ ] **Task Cycling** - Auto-switch between tasks
- [ ] **Persistence** - Save/restore state across restarts
- [ ] **Events** - Domain event publishing and handling

## TDD Process Guide

For each test, follow the Red-Green-Refactor cycle:

1. **RED** 🔴 - Write the test first (it should fail)
2. **GREEN** 🟢 - Write minimal code to make it pass
3. **REFACTOR** 🔵 - Clean up the implementation
4. **VERIFY** ✅ - Run all tests to ensure nothing broke

## Test Infrastructure Setup

```pseudo
// Test Context Builder
setup_test_context():
    db = create_in_memory_sqlite()
    event_bus = MemEventBus::new()
    
    // Repositories
    task_repo = SqliteTaskRepository::new(db)
    config_repo = SqliteConfigRepository::new(db)
    timer_repo = SqliteTimerRepository::new(db)
    
    // Services
    timer_service = TimerService::new(timer_repo)
    task_cycler = TaskCyclerService::new(task_repo)
    audio_service = MockAudioService::new()
    
    // Use Cases
    bootstrap_usecases(repos, services, event_bus)
    
    return TestContext {
        db, repos, services, usecases, event_bus
    }

// Test Helper Functions
create_test_task(title, status = "pending"):
    return Task::builder()
        .title(title)
        .status(status)
        .build()

start_timer_with_task(context, task_id):
    context.usecases.start_timer.execute(task_id)

advance_time_by(context, seconds):
    for i in 0..seconds:
        context.timer_service.tick()

assert_event_published(context, event_type):
    events = context.event_bus.get_published_events()
    assert events.contains(event_type)

get_timer_state(context):
    return context.usecases.get_timer_state.execute()
```

---

## Phase 1: Timer Core Tests (Tests 1-5)
*Goal: Establish basic timer functionality*

### Test 1: Timer initializes in idle state
```pseudo
TEST: "timer_should_initialize_in_idle_state"
GIVEN: 
    context = setup_test_context()
WHEN:  
    state = get_timer_state(context)
THEN:  
    assert state.status == TimerStatus::Idle
    assert state.phase == Phase::Work
    assert state.duration == 25 * 60  // 25 minutes default
```

### Test 2: Timer can start from idle
```pseudo
TEST: "timer_should_start_from_idle_state"
GIVEN: 
    context = setup_test_context()
    initial_state = get_timer_state(context)
    assert initial_state.status == TimerStatus::Idle
WHEN:  
    result = context.usecases.start_timer.execute(None)
THEN:  
    assert result.is_ok()
    state = get_timer_state(context)
    assert state.status == TimerStatus::Running
    assert_event_published(context, "TimerStarted")
```

### Test 3: Timer cannot start when already running
```pseudo
TEST: "timer_should_not_start_when_already_running"
GIVEN: 
    context = setup_test_context()
    context.usecases.start_timer.execute(None)
    assert get_timer_state(context).status == TimerStatus::Running
WHEN:  
    result = context.usecases.start_timer.execute(None)
THEN:  
    assert result.is_err()
    assert result.error == "Timer already running"
```

### Test 4: Timer can pause when running
```pseudo
TEST: "timer_should_pause_when_running"
GIVEN: 
    context = setup_test_context()
    context.usecases.start_timer.execute(None)
    advance_time_by(context, 60)  // 1 minute
WHEN:  
    result = context.usecases.pause_timer.execute()
THEN:  
    assert result.is_ok()
    state = get_timer_state(context)
    assert state.status == TimerStatus::Paused
    assert state.duration == 24 * 60  // 24 minutes remaining
    assert_event_published(context, "TimerPaused")
```

### Test 5: Timer resets to initial state
```pseudo
TEST: "timer_should_reset_to_initial_state"
GIVEN: 
    context = setup_test_context()
    context.usecases.start_timer.execute(None)
    advance_time_by(context, 300)  // 5 minutes
WHEN:  
    result = context.usecases.reset_timer.execute()
THEN:  
    assert result.is_ok()
    state = get_timer_state(context)
    assert state.status == TimerStatus::Idle
    assert state.duration == 25 * 60  // Back to 25 minutes
    assert_event_published(context, "TimerReset")
```

---

## Phase 2: Task Basics Tests (Tests 6-10)
*Goal: Implement basic task management*

### Test 6: Create task with title
```pseudo
TEST: "should_create_task_with_title"
GIVEN: 
    context = setup_test_context()
    request = CreateTaskRequest { 
        title: "Write integration tests",
        description: None 
    }
WHEN:  
    result = context.usecases.create_task.execute(request)
THEN:  
    assert result.is_ok()
    task = result.value
    assert task.title == "Write integration tests"
    assert task.status == TaskStatus::Pending
    assert task.sessions_completed == 0
    assert_event_published(context, "TaskCreated")
```

### Test 7: Task has unique ID
```pseudo
TEST: "tasks_should_have_unique_ids"
GIVEN: 
    context = setup_test_context()
WHEN:  
    task1 = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Task 1" }
    )
    task2 = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Task 2" }
    )
THEN:  
    assert task1.value.id != task2.value.id
    assert task1.value.id.is_valid_uuid()
    assert task2.value.id.is_valid_uuid()
```

### Test 8: Find task by ID
```pseudo
TEST: "should_find_task_by_id"
GIVEN: 
    context = setup_test_context()
    created_task = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Find me" }
    ).value
WHEN:  
    result = context.usecases.get_task.execute(created_task.id)
THEN:  
    assert result.is_ok()
    found_task = result.value
    assert found_task.id == created_task.id
    assert found_task.title == "Find me"
```

### Test 9: Update task status
```pseudo
TEST: "should_update_task_status"
GIVEN: 
    context = setup_test_context()
    task = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Update my status" }
    ).value
    assert task.status == TaskStatus::Pending
WHEN:  
    result = context.usecases.update_task.execute(
        task.id,
        UpdateTaskRequest { 
            status: Some(TaskStatus::InProgress),
            title: None,
            description: None
        }
    )
THEN:  
    assert result.is_ok()
    updated_task = result.value
    assert updated_task.status == TaskStatus::InProgress
    assert_event_published(context, "TaskStatusChanged")
```

### Test 10: Delete task
```pseudo
TEST: "should_delete_task"
GIVEN: 
    context = setup_test_context()
    task = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Delete me" }
    ).value
WHEN:  
    delete_result = context.usecases.delete_task.execute(task.id)
    find_result = context.usecases.get_task.execute(task.id)
THEN:  
    assert delete_result.is_ok()
    assert find_result.is_err()
    assert find_result.error == "Task not found"
    assert_event_published(context, "TaskDeleted")
```

---

## Phase 3: Timer-Task Integration Tests (Tests 11-15)
*Goal: Connect timer and task functionality*

### Test 11: Start timer with task
```pseudo
TEST: "should_start_timer_with_specific_task"
GIVEN: 
    context = setup_test_context()
    task = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Focus task" }
    ).value
WHEN:  
    result = context.usecases.start_timer.execute(Some(task.id))
THEN:  
    assert result.is_ok()
    state = get_timer_state(context)
    assert state.status == TimerStatus::Running
    assert state.active_task_id == Some(task.id)
    assert_event_published(context, "WorkSessionStarted")
```

### Test 12: Complete work session increments task counter
```pseudo
TEST: "completing_work_session_should_increment_task_counter"
GIVEN: 
    context = setup_test_context()
    task = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Track my sessions" }
    ).value
    context.usecases.start_timer.execute(Some(task.id))
WHEN:  
    // Fast forward through entire work session
    advance_time_by(context, 25 * 60)  // 25 minutes
THEN:  
    updated_task = context.usecases.get_task.execute(task.id).value
    assert updated_task.sessions_completed == 1
    assert_event_published(context, "WorkSessionCompleted")
    assert_event_published(context, "TaskSessionCompleted")
    
    // Timer should transition to break
    state = get_timer_state(context)
    assert state.phase == Phase::Break
```

### Test 13: Timer state persists across restarts
```pseudo
TEST: "timer_state_should_persist_across_restarts"
GIVEN: 
    context = setup_test_context()
    context.usecases.start_timer.execute(None)
    advance_time_by(context, 10 * 60)  // 10 minutes
    state_before = get_timer_state(context)
WHEN:  
    // Simulate app restart
    new_context = setup_test_context_with_same_db(context.db)
    state_after = get_timer_state(new_context)
THEN:  
    assert state_after.status == state_before.status
    assert state_after.duration == state_before.duration
    assert state_after.phase == state_before.phase
```

### Test 14: Events published on state changes
```pseudo
TEST: "should_publish_events_on_all_state_changes"
GIVEN: 
    context = setup_test_context()
    task = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Event test" }
    ).value
WHEN:  
    context.usecases.start_timer.execute(Some(task.id))
    context.usecases.pause_timer.execute()
    context.usecases.start_timer.execute(Some(task.id))
    context.usecases.skip_phase.execute()
    context.usecases.reset_timer.execute()
THEN:  
    events = context.event_bus.get_published_events()
    assert events.contains("TimerStarted")
    assert events.contains("TimerPaused")
    assert events.contains("PhaseSkipped")
    assert events.contains("TimerReset")
    assert events.count() >= 5
```

### Test 15: Task queue returns next task
```pseudo
TEST: "task_queue_should_return_next_incomplete_task"
GIVEN: 
    context = setup_test_context()
    task1 = create_and_complete_task(context, "Completed task")
    task2 = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Pending task" }
    ).value
    task3 = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Another pending" }
    ).value
WHEN:  
    next_task = context.usecases.get_task_queue.execute().value.first()
THEN:  
    assert next_task.is_some()
    assert next_task.value.id == task2.id  // First pending task
    assert next_task.value.status == TaskStatus::Pending
```

---

## Phase 4: Configuration Tests (Tests 16-20)
*Goal: Implement configuration management*

### Test 16: Load default configuration
```pseudo
TEST: "should_load_default_configuration"
GIVEN: 
    context = setup_test_context()
WHEN:  
    config = context.usecases.get_config.execute().value
THEN:  
    assert config.task_defaults.work_duration == 25 * 60
    assert config.task_defaults.short_break == 5 * 60
    assert config.task_defaults.long_break == 15 * 60
    assert config.general.theme == "light"
    assert config.audio.volume == 0.5
```

### Test 17: Update timer durations
```pseudo
TEST: "should_update_timer_durations_in_config"
GIVEN: 
    context = setup_test_context()
    config = context.usecases.get_config.execute().value
WHEN:  
    config.task_defaults.work_duration = 30 * 60  // 30 minutes
    config.task_defaults.short_break = 10 * 60    // 10 minutes
    result = context.usecases.update_config.execute(config)
THEN:  
    assert result.is_ok()
    updated_config = context.usecases.get_config.execute().value
    assert updated_config.task_defaults.work_duration == 30 * 60
    assert updated_config.task_defaults.short_break == 10 * 60
    assert_event_published(context, "ConfigUpdated")
```

### Test 18: Config changes apply to new sessions
```pseudo
TEST: "config_changes_should_apply_to_new_timer_sessions"
GIVEN: 
    context = setup_test_context()
    config = context.usecases.get_config.execute().value
    config.task_defaults.work_duration = 20 * 60  // 20 minutes
    context.usecases.update_config.execute(config)
WHEN:  
    context.usecases.start_timer.execute(None)
    state = get_timer_state(context)
THEN:  
    assert state.duration == 20 * 60  // Uses new config
    assert state.phase == Phase::Work
```

### Test 19: Reset config to defaults
```pseudo
TEST: "should_reset_config_to_factory_defaults"
GIVEN: 
    context = setup_test_context()
    config = context.usecases.get_config.execute().value
    config.task_defaults.work_duration = 45 * 60
    config.general.theme = "dark"
    context.usecases.update_config.execute(config)
WHEN:  
    result = context.usecases.reset_config.execute()
THEN:  
    assert result.is_ok()
    reset_config = result.value
    assert reset_config.task_defaults.work_duration == 25 * 60
    assert reset_config.general.theme == "light"
    assert_event_published(context, "ConfigReset")
```

### Test 20: Validate config boundaries
```pseudo
TEST: "should_validate_config_boundaries"
GIVEN: 
    context = setup_test_context()
    config = context.usecases.get_config.execute().value
WHEN:  
    // Try invalid values
    config.audio.volume = 1.5  // Over max
    result1 = context.usecases.update_config.execute(config.clone())
    
    config.audio.volume = -0.1  // Below min
    result2 = context.usecases.update_config.execute(config.clone())
    
    config.task_defaults.work_duration = 0  // Invalid duration
    result3 = context.usecases.update_config.execute(config)
THEN:  
    assert result1.is_err()
    assert result2.is_err()
    assert result3.is_err()
```

---

## Phase 5: Advanced Workflow Tests (Tests 21-30)
*Goal: Test complex user scenarios*

### Test 21: Complete full pomodoro cycle
```pseudo
TEST: "should_complete_full_pomodoro_cycle"
GIVEN: 
    context = setup_test_context()
    task = context.usecases.create_task.execute(
        CreateTaskRequest { title: "Full cycle test" }
    ).value
WHEN:  
    // Work session
    context.usecases.start_timer.execute(Some(task.id))
    advance_time_by(context, 25 * 60)
    
    // Short break (auto-started)
    advance_time_by(context, 5 * 60)
    
    // Another work session
    context.usecases.start_timer.execute(Some(task.id))
    advance_time_by(context, 25 * 60)
THEN:  
    task = context.usecases.get_task.execute(task.id).value
    assert task.sessions_completed == 2
    
    events = context.event_bus.get_published_events()
    assert events.count("WorkSessionCompleted") == 2
    assert events.count("BreakSessionCompleted") == 1
```

### Test 22: Task cycling with multiple tasks
```pseudo
TEST: "should_cycle_through_incomplete_tasks"
GIVEN: 
    context = setup_test_context()
    task1 = create_task(context, "Task 1", TaskStatus::InProgress)
    task2 = create_task(context, "Task 2", TaskStatus::Pending)
    task3 = create_task(context, "Task 3", TaskStatus::Completed)
    
    // Set cycling strategy
    strategy = TaskCyclingStrategy::Sequential
WHEN:  
    next1 = context.usecases.cycle_task.execute(strategy, None).value
    next2 = context.usecases.cycle_task.execute(strategy, Some(next1.id)).value
    next3 = context.usecases.cycle_task.execute(strategy, Some(next2.id)).value
THEN:  
    assert next1.id == task1.id  // First incomplete
    assert next2.id == task2.id  // Second incomplete
    assert next3.id == task1.id  // Cycles back (task3 is completed)
```

### Test 23: Skip phase during work session
```pseudo
TEST: "should_skip_from_work_to_break_phase"
GIVEN: 
    context = setup_test_context()
    task = create_task(context, "Skip test")
    context.usecases.start_timer.execute(Some(task.id))
    advance_time_by(context, 10 * 60)  // 10 minutes into work
WHEN:  
    result = context.usecases.skip_phase.execute()
THEN:  
    assert result.is_ok()
    state = get_timer_state(context)
    assert state.phase == Phase::Break
    assert state.duration == 5 * 60  // Full break duration
    assert_event_published(context, "PhaseSkipped")
    
    // Task should NOT get session credit
    task = context.usecases.get_task.execute(task.id).value
    assert task.sessions_completed == 0
```

### Test 24: Pause and resume maintains state
```pseudo
TEST: "pause_and_resume_should_maintain_timer_state"
GIVEN: 
    context = setup_test_context()
    task = create_task(context, "Pause test")
    context.usecases.start_timer.execute(Some(task.id))
    advance_time_by(context, 10 * 60)  // 10 minutes
WHEN:  
    state_before_pause = get_timer_state(context)
    context.usecases.pause_timer.execute()
    
    // Simulate time passing while paused
    advance_time_by(context, 5 * 60)  
    
    context.usecases.start_timer.execute(Some(task.id))
    state_after_resume = get_timer_state(context)
THEN:  
    // Duration shouldn't change while paused
    assert state_before_pause.duration == 15 * 60  // 15 min remaining
    assert state_after_resume.duration == 15 * 60  // Still 15 min
    assert state_after_resume.active_task_id == Some(task.id)
```

### Test 25: Task with custom timer settings
```pseudo
TEST: "task_with_custom_settings_overrides_defaults"
GIVEN: 
    context = setup_test_context()
    task = create_task(context, "Custom timer task")
    
    // Apply custom settings to task
    custom_settings = TaskSettings {
        work_duration: Some(15 * 60),  // 15 minutes
        short_break: Some(3 * 60),     // 3 minutes
        long_break: None
    }
    context.usecases.update_task_settings.execute(task.id, custom_settings)
WHEN:  
    context.usecases.start_timer.execute(Some(task.id))
    state = get_timer_state(context)
THEN:  
    assert state.duration == 15 * 60  // Uses task's custom duration
    assert state.phase == Phase::Work
    
    // Complete work and check break duration
    advance_time_by(context, 15 * 60)
    state = get_timer_state(context)
    assert state.phase == Phase::Break
    assert state.duration == 3 * 60  // Custom break duration
```

### Test 26: Search and filter tasks
```pseudo
TEST: "should_search_and_filter_tasks"
GIVEN: 
    context = setup_test_context()
    task1 = create_task(context, "Write unit tests", TaskStatus::InProgress)
    task2 = create_task(context, "Write documentation", TaskStatus::Pending)
    task3 = create_task(context, "Review code", TaskStatus::Completed)
    task4 = create_task(context, "Deploy to production", TaskStatus::Pending)
WHEN:  
    // Search by text
    search_results = context.usecases.search_tasks.execute(
        "Write", None
    ).value
    
    // Filter by status
    pending_tasks = context.usecases.search_tasks.execute(
        "", Some(TaskStatus::Pending)
    ).value
    
    // Combined search and filter
    filtered_search = context.usecases.search_tasks.execute(
        "Write", Some(TaskStatus::Pending)
    ).value
THEN:  
    assert search_results.len() == 2  // "Write unit tests" and "Write documentation"
    assert pending_tasks.len() == 2  // task2 and task4
    assert filtered_search.len() == 1  // Only "Write documentation" (pending)
```

### Test 27: Long break after multiple sessions
```pseudo
TEST: "should_trigger_long_break_after_4_work_sessions"
GIVEN: 
    context = setup_test_context()
    task = create_task(context, "Long break test")
    
    // Complete 3 work sessions
    for i in 0..3:
        context.usecases.start_timer.execute(Some(task.id))
        advance_time_by(context, 25 * 60)  // Work
        advance_time_by(context, 5 * 60)   // Short break
WHEN:  
    // Start 4th work session
    context.usecases.start_timer.execute(Some(task.id))
    advance_time_by(context, 25 * 60)  // Complete 4th work
    
    state = get_timer_state(context)
THEN:  
    assert state.phase == Phase::Break
    assert state.duration == 15 * 60  // Long break (15 min)
    assert_event_published(context, "LongBreakStarted")
```

### Test 28: Switch active task mid-session
```pseudo
TEST: "should_switch_active_task_during_timer_session"
GIVEN: 
    context = setup_test_context()
    task1 = create_task(context, "Original task")
    task2 = create_task(context, "New task")
    
    context.usecases.start_timer.execute(Some(task1.id))
    advance_time_by(context, 10 * 60)  // 10 minutes
WHEN:  
    result = context.usecases.switch_timer_task.execute(task2.id)
THEN:  
    assert result.is_ok()
    state = get_timer_state(context)
    assert state.active_task_id == Some(task2.id)
    assert state.duration == 15 * 60  // Timer continues
    assert state.status == TimerStatus::Running
    
    // Complete session
    advance_time_by(context, 15 * 60)
    
    // Only task2 should get credit
    task1_updated = context.usecases.get_task.execute(task1.id).value
    task2_updated = context.usecases.get_task.execute(task2.id).value
    assert task1_updated.sessions_completed == 0
    assert task2_updated.sessions_completed == 1
```

### Test 29: Handle timer tick events
```pseudo
TEST: "should_emit_tick_events_every_second"
GIVEN: 
    context = setup_test_context()
    context.usecases.start_timer.execute(None)
    context.event_bus.clear_events()
WHEN:  
    // Advance 5 seconds
    for i in 0..5:
        context.timer_service.tick()
THEN:  
    events = context.event_bus.get_published_events()
    tick_events = events.filter(|e| e.type == "TimerTick")
    assert tick_events.count() == 5
    
    // Check tick event data
    last_tick = tick_events.last()
    assert last_tick.data.remaining == (25 * 60) - 5
    assert last_tick.data.phase == Phase::Work
```

### Test 30: Complete end-to-end workflow
```pseudo
TEST: "complete_productivity_workflow_integration"
GIVEN: 
    context = setup_test_context()
    
    // Create multiple tasks
    tasks = [
        create_task(context, "Email responses"),
        create_task(context, "Code review"),
        create_task(context, "Write tests"),
    ]
    
    // Configure shorter durations for testing
    config = context.usecases.get_config.execute().value
    config.task_defaults.work_duration = 2 * 60  // 2 min
    config.task_defaults.short_break = 1 * 60   // 1 min
    context.usecases.update_config.execute(config)
WHEN:  
    // Work on first task
    context.usecases.start_timer.execute(Some(tasks[0].id))
    advance_time_by(context, 2 * 60)
    
    // Break
    advance_time_by(context, 1 * 60)
    
    // Cycle to next task
    next_task = context.usecases.cycle_task.execute(
        TaskCyclingStrategy::Sequential, 
        Some(tasks[0].id)
    ).value
    
    // Work on second task
    context.usecases.start_timer.execute(Some(next_task.id))
    advance_time_by(context, 1 * 60)  // Partial session
    
    // Pause and switch tasks manually
    context.usecases.pause_timer.execute()
    context.usecases.switch_timer_task.execute(tasks[2].id)
    context.usecases.start_timer.execute(Some(tasks[2].id))
    advance_time_by(context, 1 * 60)  // Complete remaining time
    
    // Mark task as completed
    context.usecases.update_task.execute(
        tasks[2].id,
        UpdateTaskRequest { status: Some(TaskStatus::Completed) }
    )
THEN:  
    // Verify task states
    task1 = context.usecases.get_task.execute(tasks[0].id).value
    task2 = context.usecases.get_task.execute(tasks[1].id).value
    task3 = context.usecases.get_task.execute(tasks[2].id).value
    
    assert task1.sessions_completed == 1
    assert task2.sessions_completed == 0  // Didn't complete
    assert task3.sessions_completed == 1
    assert task3.status == TaskStatus::Completed
    
    // Verify events
    events = context.event_bus.get_published_events()
    assert events.contains("WorkSessionCompleted")
    assert events.contains("BreakSessionCompleted")
    assert events.contains("TaskStatusChanged")
    assert events.contains("TimerPaused")
    
    // Verify next task in queue excludes completed
    queue = context.usecases.get_task_queue.execute().value
    assert !queue.contains(|t| t.id == tasks[2].id)
```

---

## Test Execution Strategy

### Order of Implementation
1. **Core First**: Implement Phase 1 tests first (Timer Core)
2. **Build Up**: Each phase depends on the previous
3. **Integration Last**: Phase 5 tests verify everything works together

### Running Tests
```pseudo
// Run all tests
cargo test --package infra --test integration

// Run specific phase
cargo test --package infra --test integration phase_1

// Run with verbose output
cargo test --package infra --test integration -- --nocapture

// Run single test
cargo test --package infra --test integration test_1
```

### Debugging Failed Tests
1. Check test output for assertion details
2. Verify test setup/context is correct
3. Add debug prints to see intermediate state
4. Run test in isolation to eliminate interference
5. Check event bus for unexpected events

## Success Criteria

Each test should:
- ✅ Be independent and isolated
- ✅ Test one specific behavior
- ✅ Have clear, descriptive names
- ✅ Follow Given/When/Then structure
- ✅ Clean up after itself
- ✅ Run quickly (< 100ms per test)

## Next Steps After MVP

Once all 30 tests pass:
1. Add audio integration tests
2. Add notification system tests
3. Add import/export functionality tests
4. Add concurrent user scenario tests
5. Add error recovery tests
6. Add performance/stress tests