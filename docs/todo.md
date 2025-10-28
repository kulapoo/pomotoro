Detailed Refactoring Plan: Clean Timer Architecture with Configuration-Based Break Scheduling

Problem Context

The current system has three redundant session tracking locations:

1. Task::current_sessions - Tracks task progress (needed)
2. TimerState::session_count - Duplicated across all state variants (redundant)
3. TimerState::entity_session_count - In Working/ShortBreak/LongBreak (redundant)

Additionally, TimerConfiguration is duplicated in every TimerState variant instead of being stored once at
the Timer level.

Key Insight

You're right - sessions_until_long_break is a configuration item, not task-specific business data. It defines
how the timer behaves, so it belongs in TimerConfiguration.

Final Architecture

// Timer: Owns configuration and provides countdown service
struct Timer {
id: TimerId,
configuration: TimerConfiguration, // Stored once
state: TimerState, // Pure state machine
}

// TimerConfiguration: Complete timer behavior settings
struct TimerConfiguration {
work_duration: Duration,
short_break_duration: Duration,
long_break_duration: Duration,
sessions_until_long_break: u8, // Stays here - it's configuration!
}

// TimerState: Minimal state tracking
enum TimerState {
Idle,
Working { remaining_seconds: u32 },
ShortBreak { remaining_seconds: u32 },
LongBreak { remaining_seconds: u32 },
Paused {
paused_from: Box<TimerState>,
remaining_seconds: u32
},
}

// Task: Only tracks progress, not timer configuration
struct Task {
id: TaskId,
timer_id: TimerId,
name: String,
current_sessions: u8, // Progress toward completion
max_sessions: u8, // When task completes
// NO timer configuration here
}

How Break Determination Works

The use case layer orchestrates:

1. Get Task's current_sessions
2. Get Timer's configuration.sessions_until_long_break
3. Calculate: (task.current_sessions + 1) % timer.configuration.sessions_until_long_break == 0
4. Tell Timer which phase to start next

Example Flow

// In use case: complete_work_session.rs
pub async fn complete_work_session(task_id: TaskId) {
let task = task_repo.get(task_id);
let timer = timer_repo.get(task.timer_id);

      // Task increments its session
      task.current_sessions += 1;
      task_repo.save(task);

      // Determine next phase using Timer's config and Task's progress
      let next_phase = if task.current_sessions % timer.configuration.sessions_until_long_break == 0 {
          Phase::LongBreak
      } else {
          Phase::ShortBreak
      };

      // Tell timer what phase to start
      timer.start_phase(next_phase);
      timer_repo.save(timer);

}

Detailed TODO List

Phase 1: Simplify Domain Models

1. Remove configuration, session_count, entity_session_count, active_entity from TimerState

File: domain/src/timer/state_machine.rs

- Remove configuration: TimerConfiguration from all variants
- Remove session_count: u32 from all variants
- Remove entity_session_count: u32 from Working/ShortBreak/LongBreak
- Remove active_entity: Option<String> from all variants
- Keep ONLY remaining_seconds in active states

2. Move configuration to Timer struct level

File: domain/src/timer/timer.rs

- Add configuration: TimerConfiguration field to Timer struct
- Update new() constructor to accept configuration
- Update with_state() to handle configuration

3. Keep sessions_until_long_break in TimerConfiguration

File: domain/src/timer/mod.rs or configuration.rs

- Ensure TimerConfiguration has sessions_until_long_break: u8
- This stays as timer configuration, NOT moved to Task

Phase 2: Refactor State Transitions

4. Update state transitions to remove session counting logic

File: domain/src/timer/transitions.rs

- Remove ALL session counting from state transitions
- Remove break type determination from transitions
- Transitions should only handle state changes, not business logic

5. Update complete_phase to accept next phase as parameter

File: domain/src/timer/timer.rs

- Change signature: pub fn complete_phase(&mut self, next_phase: Phase) -> Result<Vec<Event>>
- Timer no longer decides what phase comes next
- Timer just transitions to the given phase

6. Update Timer to accept phase type when transitioning

File: domain/src/timer/timer.rs

- Add start_phase(&mut self, phase: Phase) -> Result<Vec<Event>>
- Timer starts the specified phase with appropriate duration from config

Phase 3: Create Use Case Orchestration

7. Create use case to determine break type using Task's current_sessions

File: usecases/src/timer/complete_work_session.rs (new)
pub async fn complete_work_session(
task_repo: &TaskRepository,
timer_repo: &TimerRepository,
task_id: TaskId,
) -> Result<()> {
// Get task and timer
// Increment task.current_sessions
// Calculate break type: current_sessions % timer.config.sessions_until_long_break
// Tell timer to start appropriate break
}

Phase 4: Simplify Events

8. Simplify timer events by removing session tracking fields

Files: domain/src/timer/events/.rs\*

- WorkPhaseCompleted: Remove session_count, task_session_count
- WorkPhaseStarted: Remove session fields
- PhaseCompleted: Simplify to just phase transition info

Phase 5: Database Updates

9. Update database schema to remove session_count from timers table

Files: infra/migrations/, infra/src/schema.rs\*

- Remove session_count column from timers table
- Keep configuration as JSON including sessions_until_long_break

10. Update database models

File: infra/src/adapters/database/models.rs

- Update TimerDb to not serialize session_count
- Ensure configuration includes sessions_until_long_break

Phase 6: Refactor Timer Methods

11. Refactor Timer methods that depend on session logic

File: domain/src/timer/timer.rs

- session_display(): Remove or make it query Task
- progress_percentage(): Use configuration from struct level
- Remove any method that tracks sessions

Phase 7: Fix Integration Points

12. Fix task switching to load new task's timer configuration

File: usecases/src/task/switch_task.rs

- When switching tasks, optionally update Timer configuration
- Each task could have preferred timer settings

13. Update UI to get session info from Task, time from Timer

Files: ui/src/pages/timer/timer_vm.rs, ui/src/pages/task/task_vm.rs

- Timer VM: Only provides remaining time
- Task VM: Provides session progress (2/5)
- Combined: "Session 2/5, 12:34 remaining"

Phase 8: Fix Compilation and Tests

14. Fix all compilation errors from removed fields

- Update all references to removed fields throughout codebase
- Fix method calls that expect session data

15. Update timer state machine tests

File: domain/src/timer/state_machine.rs (test module)

- Remove tests for session counting
- Update state creation in tests

16. Update timer transition tests

File: domain/src/timer/transitions.rs (test module)

- Remove session-based transition tests
- Add tests for phase-parameter transitions

17. Add tests for break determination logic in use case

File: usecases/src/timer/complete_work_session.rs (test module)

- Test short break after sessions 1, 2, 3
- Test long break after session 4
- Test cycle repeats

18. Test task completion still works correctly

File: usecases/src/task/complete_session.rs (tests)

- Verify tasks complete at max_sessions
- Verify progress tracking works

19. Test task switching loads correct configuration

- Test that switching tasks can update timer config
- Test that break patterns follow timer configuration

20. Run full test suite and fix failures

- Run cargo test in all crates
- Fix any remaining issues

Files to Modify

Domain Layer (8 files)

- timer/state_machine.rs
- timer/timer.rs
- timer/transitions.rs
- timer/events/work_phase_completed.rs
- timer/events/work_phase_started.rs
- timer/events/phase_completed.rs
- timer/mod.rs
- timer/configuration.rs (if separate)

Use Cases Layer (5 files)

- NEW: timer/complete_work_session.rs
- timer/start_timer_session.rs
- timer/pause_timer_session.rs
- timer/reset_timer_session.rs
- task/switch_task.rs

Infrastructure Layer (5 files)

- adapters/database/models.rs
- adapters/timer/timer_dto.rs
- schema.rs
- migrations/[new_migration].sql
- adapters/database/sqlite_timer_repository.rs

UI Layer (3 files)

- pages/timer/timer_vm.rs
- pages/task/task_vm.rs
- components/session_indicator.rs

Key Benefits

1. Single source of truth: Each piece of data lives in exactly one place
2. Timer owns timing config: Including when to take long breaks
3. Task owns progress: Only tracks current_sessions for completion
4. Use case orchestrates: Combines Task progress with Timer config
5. Clean separation: Timer handles time, Task handles progress, use case handles business logic

Critical Decision Point

Since sessions_until_long_break stays in TimerConfiguration, this means:

- All tasks using the same timer share the same break pattern
- If tasks need different break patterns, they need different timers
- OR: Timer configuration could be updated when switching tasks

This is cleaner architecturally - the timer's behavior (including break patterns) is defined by its
configuration, not by the task using it.
