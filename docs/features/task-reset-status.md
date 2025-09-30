# Task Reset Status Feature

## Overview
Added functionality to reset completed tasks back to `Queued` status, with optional session reset.

## Implementation

### Domain Layer
- **Location**: `/home/jpt/src/oss/pomotoro/domain/src/task/task.rs`
- **Exported**: `TaskPatch` struct now publicly available via `domain::TaskPatch`
- **Existing methods**:
  - `reset_sessions()` - resets to `Active` status (line 78-82)
  - `queue()`, `activate()`, `pause()` - prevented from working on `Completed` tasks

### Use Case Layer
- **File**: `/home/jpt/src/oss/pomotoro/usecases/src/task/reset_task_status.rs`
- **Function**: `reset_task_status(task_repo, task_id, reset_sessions) -> Result<()>`
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
- **Command**: `reset_task_status`
- **Tauri Endpoint**: `reset_task_status`
- **Request**:
  ```rust
  {
    task_id: String,
    reset_sessions: bool
  }
  ```
- **Response**: `TaskDto`
- **Logging**:
  - `info!` on start: "Resetting task status: id={}, reset_sessions={}"
  - `error!` on failures with context
  - `info!` on success: "Successfully reset task status: id={}, new_status={:?}"

### UI Layer
- **File**: `/home/jpt/src/oss/pomotoro/ui/src/pages/task/task_vm.rs`
- **Method**: `reset_task_to_queued(task_id: TaskId, reset_sessions: bool)`
- **Behavior**:
  - Invokes `reset_task_status` command
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
usecases::task::reset_task_status(&task_repo, task_id, true).await?;
```

## Differences from `reset_sessions`

| Feature | `reset_sessions()` | `reset_task_status()` |
|---------|-------------------|---------------------|
| Target status | `Active` | `Queued` |
| Works on completed? | Yes | Yes |
| Resets session count | Always | Optional |
| Clears completed_at | Yes | Yes |

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
- Use Case: `/home/jpt/src/oss/pomotoro/usecases/src/task/reset_task_status.rs`
- Command: `/home/jpt/src/oss/pomotoro/infra/src/commands/task_cmd.rs:223-262`
- UI: `/home/jpt/src/oss/pomotoro/ui/src/pages/task/task_vm.rs:1125-1212`
- Registration: `/home/jpt/src/oss/pomotoro/infra/src/lib.rs:146`

## Status

✅ **Implemented and Compiled Successfully**

All layers properly integrated with comprehensive error handling and logging.