# Task Reset Feature

## Overview
Functionality to reset tasks back to `Queued` status, with optional session count reset.

## Implementation

### Domain Layer
- **Location**: `/home/jpt/src/oss/pomotoro/domain/src/task/task.rs`
- **Exported**: `TaskPatch` struct now publicly available via `domain::TaskPatch`
- **Existing methods**:
  - `reset_sessions()` - resets session count to 0 and status to `Active`
  - `queue()`, `activate()`, `pause()` - prevented from working on `Completed` tasks

### Use Case Layer
- **File**: `/home/jpt/src/oss/pomotoro/usecases/src/task/reset_task.rs`
- **Function**: `reset_task(task_repo, task_id, reset_sessions) -> Result<()>`
- **Parameters**:
  - `task_id: TaskId` - The task to reset
  - `reset_sessions: bool` - Whether to reset session count to 0
- **Behavior**:
  - Resets task status to `Queued`
  - Clears `completed_at` timestamp
  - Optionally resets `current_sessions` to 0
  - Works on tasks in **any** status (including `Completed`)

### Command Layer
- **File**: `/home/jpt/src/oss/pomotoro/infra/src/commands/task_cmd.rs`
- **Command**: `reset_task`
- **Tauri Endpoint**: `reset_task`
- **Request**:
  ```rust
  {
    task_id: String,
    reset_sessions: bool
  }
  ```
- **Response**: `TaskDto`
- **Logging**:
  - `info!` on start: "Resetting task: id={}, reset_sessions={}"
  - `error!` on failures with context
  - `info!` on success: "Successfully reset task: id={}, new_status={:?}"

### UI Layer
- **File**: `/home/jpt/src/oss/pomotoro/ui/src/pages/task/task_vm.rs`
- **Method**: `reset_task_to_queued(task_id: TaskId, reset_sessions: bool)`
- **Behavior**:
  - Invokes `reset_task` command
  - Updates task list on success
  - Handles errors with console logging
  - Refetches all tasks on error to ensure consistency

## Usage Examples

### Reset task status only (keep session count)
```rust
vm.reset_task_to_queued(task_id, false);
```

### Reset task status and sessions
```rust
vm.reset_task_to_queued(task_id, true);
```

### Backend direct call
```rust
usecases::task::reset_task(&task_repo, task_id, true).await?;
```

## Testing Recommendations

1. **Unit Tests**: Test the usecase with different task statuses
2. **Integration Tests**: Test command invocation from UI
3. **Manual Testing**:
   - Create task, complete it, reset to queued
   - Verify session count with `reset_sessions=true`
   - Verify session count preserved with `reset_sessions=false`
   - Check logs propagate correctly

## Related Files

- Domain: `/home/jpt/src/oss/pomotoro/domain/src/task/task.rs`
- Use Case: `/home/jpt/src/oss/pomotoro/usecases/src/task/reset_task.rs`
- Command: `/home/jpt/src/oss/pomotoro/infra/src/commands/task_cmd/reset_task.rs`
- UI: `/home/jpt/src/oss/pomotoro/ui/src/pages/task/task_directory/task_dir_vm.rs:546-601`
- Registration: `/home/jpt/src/oss/pomotoro/infra/src/lib.rs:146`

## Status

✅ **Implemented and Compiled Successfully**

All layers properly integrated with comprehensive error handling and logging.