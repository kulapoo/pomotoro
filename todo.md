# Todo

## Automatic Task Cycling - UI Integration & Failsafe
- [ ] **Fix Timer VM: Add ACTIVE_CHANGED event listener**
  - File: `ui/src/pages/timer/timer_vm/initialization.rs`
  - Add listener for `task::ACTIVE_CHANGED` to update active task when backend auto-cycles

- [ ] **Add failsafe for "no incomplete tasks" scenario**
  - File: `infra/src/adapters/task/event_handlers/task_completed.rs`
  - When cycling returns `None`, clear timer's `active_task_id` and emit event to update UI

## Timer
- [ ] Reset Timer should stop the TICK event