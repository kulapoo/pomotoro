# Task Cycling Implementation Summary

## Overview
Implemented task cycling functionality for incomplete tasks in the Pomodoro timer application. The system now allows users to cycle through only incomplete tasks, automatically skipping completed ones.

## Features Implemented

### 1. Domain Layer Enhancements
- **Repository Methods**: Added `get_incomplete_tasks()` and `get_completed_tasks()` to TaskRepository trait
- **Cycling Service**: Enhanced CyclerService trait with:
  - `get_previous_task()`
  - `get_incomplete_task_queue()`
  - `cycle_to_next_incomplete_task()`
  - `cycle_to_previous_incomplete_task()`
  - `get_task_cycle_position()`
- **Pure Domain Logic**: Added helper methods to DefaultCyclingService:
  - `find_previous_task_round_robin()`
  - `find_task_cycle_position()`

### 2. Use Cases Layer
- **New Use Case**: `cycle_incomplete_task.rs` with:
  - `cycle_incomplete_task()` - Cycles to next/previous incomplete task
  - `get_incomplete_task_info()` - Gets current position in cycle
  - `get_task_cycle_position()` - Gets task position relative to total incomplete tasks
  - `CycleDirection` enum (Next/Previous)
  - `IncompleteCycleResult` struct with position and total count

### 3. Infrastructure Layer
- **Repository Implementations**: 
  - InMemoryTaskRepository: Implemented new methods for incomplete/completed task queries
  - FileTaskRepository: Implemented persistence-aware versions
- **Cycling Service**: 
  - StandardTaskCyclerService: Full implementation of new cycling methods
  - Bootstrap integration: Service registered in AppRegistry
- **Command Handlers**: Added Tauri commands:
  - `cycle_incomplete_task`
  - `get_task_cycle_position`
  - `get_incomplete_tasks`

### 4. UI Components
- **TaskCycleControls**: Navigation controls with:
  - Previous/Next buttons
  - Current position display ("Task 2 of 5")
  - Keyboard shortcuts (Ctrl+Tab / Ctrl+Shift+Tab)
  - Disabled state when only one task exists
- **TaskCompletionIndicator**: Visual indicator showing:
  - Progress bar for session completion
  - Completion status (✓ Complete or X/Y sessions)
  - Color-coded states (complete/incomplete/no selection)

### 5. View Model Enhancements
- **TasksViewModel**: Added:
  - `cycle_position` state tracking
  - `cycle_to_next_incomplete_task()` method
  - `cycle_to_previous_incomplete_task()` method
  - `update_cycle_position()` method
  - Integration with command invocations

## Key Implementation Details

### Task Completion Logic
Tasks are considered complete when:
- `current_sessions >= max_sessions`
- OR `status == TaskStatus::Completed`

### Cycling Behavior
- Only cycles through incomplete tasks
- Automatically skips completed tasks
- Round-robin cycling (wraps around at boundaries)
- Maintains current position tracking
- Handles edge cases:
  - No incomplete tasks
  - Single task
  - All tasks completed

### State Management
- Cycle position tracked as `(current_position, total_incomplete)`
- Updates automatically when:
  - Task is completed
  - Task is added/removed
  - Active task changes

### Keyboard Shortcuts
- **Ctrl+Tab**: Cycle to next incomplete task
- **Ctrl+Shift+Tab**: Cycle to previous incomplete task
- Works globally when timer page is active

## Usage Example

```rust
// In timer page component
let handle_next_task = move || {
    tasks_vm.with_value(|v| v.cycle_to_next_incomplete_task());
};

<TaskCycleControls 
    on_next=handle_next_task
    on_previous=handle_previous_task
    position=cycle_position
    is_active=is_timer_active
/>
```

## Testing
The implementation includes comprehensive tests for:
- Round-robin cycling logic
- Incomplete task filtering
- Cycle position calculation
- Edge case handling (empty lists, single task, all completed)

## Files Modified/Created

### Domain Layer
- `/domain/src/task/repository.rs` - Added incomplete/completed query methods
- `/domain/src/task/cycling_service.rs` - Enhanced cycling service trait and logic

### Use Cases Layer
- `/usecases/src/task/cycle_incomplete_task.rs` - New use case implementation
- `/usecases/src/task/mod.rs` - Module exports

### Infrastructure Layer
- `/infra/src/adapters/task/memory_repo.rs` - Repository implementation
- `/infra/src/adapters/task/file_repo.rs` - File repository implementation
- `/infra/src/adapters/task/cycling_srv.rs` - Cycling service implementation
- `/infra/src/commands/task_cmd.rs` - Command handlers
- `/infra/src/bootstrap.rs` - Service registration
- `/infra/src/lib.rs` - Command registration

### UI Layer
- `/ui/src/components/task_cycle_controls.rs` - Cycle navigation component
- `/ui/src/components/task_completion_indicator.rs` - Completion status component
- `/ui/src/pages/task/task_vm.rs` - View model enhancements
- `/ui/src/pages/timer/timer_with_cycling.rs` - Example integration
- `/ui/styles/task-cycling.css` - Component styles

### Configuration
- `/domain/src/event_names/commands.rs` - Added command names

## Integration Points
The implementation integrates seamlessly with:
- Existing timer functionality
- Task completion tracking
- Event-driven architecture
- Tauri command system
- Clean Architecture layers