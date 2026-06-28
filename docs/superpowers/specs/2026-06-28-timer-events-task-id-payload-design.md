# Timer UI events: include `task_id` in payload

**Date:** 2026-06-28
**Status:** Proposed
**Scope:** domain, infra (timer/audio/notifications handlers), react-ui

## Problem

The four timer UI events
(`timer:timer_started`, `timer:timer_paused`, `timer:timer_reset`,
`timer:timer_resumed`) emit only `TimerStateData`
(`{ state, remaining_seconds?, paused_from? }`) — no `task_id`.

On the frontend, `EventBus.ts` routes them to `applyTimerState`, which
**preserves the existing `timer.task_id`** instead of replacing it:

```ts
applyTimerState: (state) => {
  const timer = get().timer
  set({ timer: { ...timer, task_id: timer.task_id, state } })
},
```

When a task switch happens out-of-band (e.g. the orchestrator swaps the
bound task without the frontend re-fetching), every subsequent
`timer:*` event keeps the stale `task_id`. The UI then dispatches timer
commands against the wrong task.

The domain events already carry `task_id` (`TimerStarted`,
`TimerReset`, `TimerPaused` all have `task_id: TaskId`); the handlers
just discard it when serializing.

## Additional gap: `timer:timer_resumed` is an orphan

The constant `ui_listeners::timer::RESUME = "timer:timer_resumed"`
exists and the frontend listens for it, but no backend handler emits
it. `transitions::resume()` currently emits a `Started` event, so on
resume the frontend actually receives `timer:timer_started` (via
`TimerStartedHandler`). The `timer:timer_resumed` listener is dead
code.

Two side effects ride on resume today via the misnamed `Started`
event:

- `TimerStartedAudioHandler` plays a "session started" sound on resume.
- `TimerStartedNotificationHandler` posts a "SessionStarted" OS
  notification on resume.

## Goals

1. Emit `{ task_id, state }` on all four timer UI events so the
   frontend can update `timer.task_id` directly from the payload.
2. Make `timer:timer_resumed` a real, distinct event with its own
   domain semantics ("resume", not "start").
3. Preserve audio/notification behavior on resume (still fire on
   resume, but driven by the correctly-named event).
4. Lock the new `TimerResumedHandler` under the same tick-loop
   ownership invariant as the other UI-only handlers.

## Non-goals

- Changing the shape of `timer:status_changed` (it already carries
  `task_id` via `TimerStatusChangedPayload`; pre-existing
  serialization mismatch between the handler emitting `state` and the
  frontend typing it as `TimerStatusChangedPayload` is left alone).
- Touching `timer:tick` (already carries `task_id`).

## Design

### Payload shape

All four timer UI events (`timer:timer_started`, `timer:timer_paused`,
`timer:timer_reset`, `timer:timer_resumed`) emit JSON matching the
frontend `Timer` interface:

```json
{ "task_id": "<uuid>", "state": { "state": "Working", "remaining_seconds": 1234 } }
```

The `task_id` comes from the domain event payload (each timer domain
event already carries `task_id: TaskId`); the `state` object comes
from `TimerTickService::with_timer(|t| t.state())` as today.

This mirrors the existing `Timer` shape used by the
`get_timer_state` IPC command and lets the frontend switch from
`applyTimerState` (preserves task_id) to `applyTimer` (replaces the
whole timer).

### Domain layer

#### New event: `Resumed`

New file `core/domain/src/timer/events/timer_resumed.rs`. Structurally
identical to `Started` (same fields, same `Event` impl pattern), with
`event_type() == "Resumed"`:

```rust
pub struct Resumed {
    pub task_id: TaskId,
    pub phase: Phase,
    pub duration_seconds: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}
```

Registered in `core/domain/src/timer/events/mod.rs` and re-exported as
`TimerResumed` from `core/domain/src/lib.rs` alongside `TimerStarted`.

#### `transitions::resume()` emits `Resumed`

`core/domain/src/timer/transitions.rs::resume()` currently builds a
`vec![Box::new(Started::new(...))]`. Change to
`vec![Box::new(Resumed::new(...))]`. All other call sites of
`Started::new` stay as-is (start transition, post-break auto-start,
post-skip auto-start).

### Infra layer — timer event handlers

#### `TimerResumedHandler` (new)

New file
`core/infra/src/adapters/timer/event_handlers/timer_resumed.rs`,
modeled on `TimerStartedHandler`:

- `subscribes_to()` → `TypeId::of::<domain::TimerResumed>()`.
- `handle()` reads `t.state()` via `with_timer`, builds
  `{ task_id, state }` JSON (using `event.task_id`), emits
  `ui_listeners::timer::RESUME` then `STATUS_CHANGED`.
- Read-only — does **not** touch `cancel_handle` (same contract as
  the other timer UI handlers).

Registered in `event_handlers/mod.rs`, `registry.rs::register_timer_handlers`
(subscribe), and `unregister_timer_handlers`
(`clear_handlers_for_type(TypeId::of::<TimerResumedHandler>())`).

#### Update `TimerStartedHandler`, `TimerResetHandler`, `TimerPausedHandler`

Same structural change in each handler: build the combined
`{ task_id, state }` JSON instead of bare `state`, and emit it for both
the specific event (`START` / `RESET` / `PAUSE`) and `STATUS_CHANGED`.

The `task_id` is taken from the domain event payload
(`event.task_id`), not from `with_timer`, because the event is the
authoritative source of "what just happened" and is race-free relative
to concurrent task switches.

### Infra layer — audio handlers

To preserve "play a sound on resume" behavior, the audio adapter must
react to `Resumed`. Since `EventHandler::subscribes_to` returns a
single `TypeId`, add a parallel handler rather than overloading the
existing one:

1. Extract the body of `TimerStartedAudioHandler::handle` into a free
   function in `core/infra/src/adapters/audio/event_handlers.rs`:

   ```rust
   async fn play_phase_start_audio(
       audio_service: &AudioServiceWrapper,
       config_repository: &Arc<dyn ConfigRepository + Send + Sync>,
       phase: Phase,
   ) -> Result<()> { /* current body */ }
   ```

2. `TimerStartedAudioHandler::handle` calls it with `event.phase`.
3. New `TimerResumedAudioHandler` (in the same file) subscribes to
   `Resumed` and also calls `play_phase_start_audio`.
4. Register `TimerResumedAudioHandler` in the audio registry.

### Infra layer — notification handlers

Same pattern in `core/infra/src/adapters/notifications/event_handlers.rs`:

1. Extract the body of `TimerStartedNotificationHandler::handle` into
   a free function `send_session_started_notification(...)` taking
   `task_id` (it already looks up the task name from the repo).
2. New `TimerResumedNotificationHandler` subscribes to `Resumed` and
   calls the helper.
3. Register in `register_notification_handlers`.

### Frontend

#### `apps/react-ui/src/lib/tauri.ts`

Update `EventPayloadMap` entries from `TimerStateData` to `Timer`:

```ts
'timer:timer_reset': Timer,
'timer:timer_started': Timer,
'timer:timer_paused': Timer,
'timer:timer_resumed': Timer,
```

(`Timer` is already imported.)

#### `apps/react-ui/src/app/EventBus.ts`

Switch the four listeners from `applyTimerState` to `applyTimer`:

```ts
onEvent(events.timerReset, applyTimer),
onEvent(events.timerPaused, applyTimer),
onEvent(events.timerStarted, applyTimer),
onEvent(events.timerResumed, applyTimer),
```

`applyTimerState` stays (still used by `timerPhaseCompleted`).

### Tests

#### Domain unit tests (`core/domain/src/timer/events/mod.rs`)

Add a `Resumed` instance to the existing
`should_have_correct_event_types` test (assert `event_type() ==
"Resumed"`) and a serialization round-trip test mirroring
`should_serialize_timer_started_event`.

#### Infra integration tests

`core/infra/tests/app/tick_loop_invariants.rs::domain_events_do_not_mutate_tick_loop`:
add a `TimerResumed` event to the published set, locking the new
`TimerResumedHandler` under the same invariant.

`core/infra/tests/app/timer.rs`: extend the resume test (or add one)
to assert that `resume_timer_phase` produces a `TimerResumed` domain
event (and **not** `TimerStarted`), and that the frontend-facing
`timer:timer_resumed` event is emitted with a `task_id` field in its
payload.

#### Frontend

No new test infra. Verify by `tsc` + manual smoke test:

1. Start a work session, observe `timer.task_id` is set from the
   event payload (not from the pre-existing store value).
2. Pause/resume — `task_id` persists correctly through
   `timer:timer_resumed` rather than being dropped.
3. Reset — `task_id` follows the event payload.

## Verification plan

After implementation:

```bash
# Domain + infra
cargo test -p domain
cargo test -p infra
cargo test -p usecases

# Lints / typechecks
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Frontend
pnpm --filter react-ui typecheck
pnpm --filter react-ui lint
```

Manual smoke: start → pause → resume → reset, confirm
`timer.task_id` updates correctly in DevTools and survives a task
switch initiated from outside the timer page.

## Risks and mitigations

| Risk                                                                              | Mitigation                                                                              |
| --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| Resume no longer fires `Started` — other consumers exist that weren't found by grep | Searched `Started::new`, `TimerStarted`, `TypeId::of::<domain::TimerStarted>`; only the three documented subscribers exist. `cargo test` across the workspace will catch any regression. |
| Pre-existing `STATUS_CHANGED` payload/typing mismatch surfaces                    | Out of scope; left untouched.                                                           |
| Frontend behavior change: `applyTimer` overwrites `task_id` where `applyTimerState` preserved it | Intended — that's the fix. Existing call sites that depended on preservation were silently buggy. |
