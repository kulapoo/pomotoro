# Task Switch Resets Timer to Idle — Design

**Date:** 2026-06-27
**Status:** Approved (pending implementation plan)
**Related:** `docs/superpowers/plans/2026-06-27-tick-loop-direct-drive.md` (tick-loop ownership contract), `tmp/architect/27-06-2026-1332-tick-loop-boundary/design.md`

## Goal

Fix the regression where switching to a different task while a timer is running leaves the timer in a stale/inconsistent state. After this change, **switching the active task always leaves the timer bound to the new task in the `Idle` state** — the in-progress pomodoro of the old task is abandoned, and the user must press Start to begin the new task's session.

## Problem

`Timer` is a singleton: `Idle | Active(ActiveTimer { task_id, state })`. The bound `task_id` **is** the "active task" — `get_active_task` reads it directly; there is no separate focus concept.

The two switch usecases rebind `task_id` but **carry the old `TimerState` over to the new task**:

- `core/usecases/src/task/switch_active_task.rs:55-71` (called by the UI):
  ```rust
  let new_timer_remaining = task.config().timer.work_duration.as_secs() as u32;
  let new_timer = domain::Timer::with_state(
      cmd.task_id,
      timer.state().clone().with_remaining_seconds(new_timer_remaining),
  );
  ```
  `with_remaining_seconds` only swaps `remaining_seconds`; it keeps the variant (`Working`/`ShortBreak`/`LongBreak`/`Paused`). So switching mid-`ShortBreak`/`Paused` makes the new task **inherit a break it never earned**, with a wrong duration. Only the `Working` path was ever tested.

- `core/usecases/src/task/switch_task.rs:86-90` (strict, blocks running switches):
  ```rust
  // Create a new timer for the new task, preserving the state from the old timer
  // This allows seamless task switching during active sessions
  let new_timer = domain::Timer::with_state(cmd.task_id, timer.state().clone());
  ```
  Preserves the entire state machine verbatim (including a `Paused` phase leak).

The user perceives this as "the timer state is still the same after switching."

## Chosen Behavior

**Stop and go Idle.** Regardless of the prior phase (`Working`/`ShortBreak`/`LongBreak`/`Paused`/`Idle`), switching the active task produces:

```
Timer::Active(ActiveTimer { task_id: <new task>, state: TimerState::Idle })
```

The new task is **active** (selected/focused) but the timer is **not running**. The user starts the new task's pomodoro explicitly.

## Design

### Change 1 — `switch_active_task` resets to Idle (the core fix)

**File:** `core/usecases/src/task/switch_active_task.rs`

The invariant "a freshly bound task starts in `Idle`" is **already encapsulated** by `Timer::new` (`core/domain/src/timer/timer.rs:118-125`, docstring: *"Create a timer bound to `task_id`, starting in the `Idle` state."*). The bug is the usecase *bypassing* that constructor with `with_state(.., old_state.with_remaining_seconds(..))`.

Replace lines 55-71 (the comment + `new_timer_remaining` + the `with_state(..., with_remaining_seconds(...))` block) with:

```rust
let new_timer = domain::Timer::new(cmd.task_id);
```

The `new_timer_remaining`/`work_duration` computation and the `with_remaining_seconds` call are deleted entirely. `task_repo.update(task)` (line 60) stays as-is.

**Tick-loop impact:** none. The orchestrator `apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs` already does `stop → usecase → load_state → conditionally start (only if is_running())`. After this change the timer is not running, so the conditional restart naturally does nothing. Consistent with the tick-loop ownership contract in `CLAUDE.md`.

### Change 2 — Same-task idempotency guard

**File:** `core/usecases/src/task/switch_active_task.rs`

The usecase already fetches the singleton timer at line 36 (`let timer = timer_repo.get().await?;`). Immediately after that fetch — before the old-task handling at lines 38-48 — add: if the requested task is already the bound active task, **early-return `Ok(())`**.

Without this, a no-op "switch" to the *currently running* task would — after Change 1 — silently **stop the running timer** and spuriously `queue()` then re-`activate()` the same task. The guard makes same-task switches a true no-op.

```rust
let timer = timer_repo.get().await?;

if timer.task_id() == Some(cmd.task_id) {
    return Ok(());
}
```

### Change 3 — Align strict `switch_task`

**File:** `core/usecases/src/task/switch_task.rs`

Replace lines 86-90 (the "preserving the state" comment + `with_state(cmd.task_id, timer.state().clone())`) with:

```rust
let new_timer = domain::Timer::new(cmd.task_id);
```

Rationale: invariant uniformity — *any* path that rebinds the active task leaves the timer in `Idle`. Divergent usecases for the same concept are a future-bug factory. `validate_task_switch`'s running-guard (lines 19-25) stays; it is an orthogonal, stricter precondition. (Switching while `Paused` still leaks its phase today, even though running is blocked — this closes that hole too.)

## Architectural Decisions (the "no"s)

1. **Do NOT clear `Timer.task_id`** (the original proposal). Because the bound `task_id` *is* the active task, `clear_task_id()` would produce `Timer::Idle` with **no active task at all** — `get_active_task` returns `None`, the UI shows nothing active. The intent ("new task active, clean timer") is expressed correctly by `Timer::new(new_task_id)` (Active + Idle), not by clearing.

2. **Do NOT emit a timer domain event (`TimerReset`) from the usecase.** `TimerResetHandler` derives its payload from the tick-service's in-memory cache (`timer_srv.with_timer(...)`), which the orchestrator only refreshes via `load_state()` *after* the usecase returns, on a detached `tokio::spawn`. A usecase-published `TimerReset` would therefore broadcast the **old** task/phase to the tray and any non-calling window — the exact stale-cache race the tick-loop ownership contract forbids. The calling window is covered by the command's `Ok(new_timer)` return; task subscribers by `TaskActiveChanged`. (If tray sync is later wanted, emit from the orchestrator after `load_state` — follow-up, out of scope here.)

3. **Do NOT decouple "active task" from `Timer.task_id`.** That is a strategic redesign (new aggregate + repository, migrate `get_active_task`, audit every `timer.task_id()` reader). This fix *improves* the coupling (bound task ⇒ Idle timer) without worsening it. Backlog note: `switch_active_task` living in `usecases/task/` while mutating the Timer aggregate is the smell of this coupling — the natural split seam if a second focus surface ever appears (YAGNI today).

4. **Layering:** the reset belongs in the **usecase**, expressed via the **existing** `Timer::new` constructor — **not** a new `ActiveTimer::switch_task` method. "Switch active task" is a task-list concern (`Task::activate`/`Task::queue` already own selection); pushing a `switch_task` onto the Timer aggregate would mislocate a task-domain responsibility. `Timer::new` self-documents the invariant; one caller needs it (YAGNI).

## Edge Cases & Invariants

| Prior state | After switch to new task | Correct? |
|---|---|---|
| `Working` (running) | `Active{new, Idle}` — timer stops | ✅ abandons old pomodoro |
| `ShortBreak` / `LongBreak` | `Active{new, Idle}` — no inherited break | ✅ (this is the regression fix) |
| `Paused` | `Active{new, Idle}` — paused phase discarded | ✅ |
| `Idle` | `Active{new, Idle}` | ✅ no-op-equivalent |
| Same task already active | unchanged (idempotency guard) | ✅ running timer not stopped |

- **Old task's partial pomodoro:** discarded, 0 session credit — correct Pomodoro semantics (an abandoned mid-work session earns no credit).
- **Old task status:** `queue()`d today on switch. Intentional for an abandoned-in-work task, but a **product question** (not architectural) whether `Paused`/another status fits better. Out of scope.

## Tests

**File:** `core/infra/tests/app/adv_timer.rs`

The existing test `should_switch_active_task_during_timer_session` (line 825) only switches during `Working` and asserts `status == Running` (lines 977-981) — exactly why the break/Paused regression escaped. It also relies on the timer staying `Running` after the switch so it can tick task2 to completion for the session-credit check.

**Update the existing test:**
- Post-switch assertions (lines 972-981): change `status == Running` → `status == Idle`; the `remaining == 25*60` assertion is no longer meaningful for an Idle timer — replace with the Idle-status assertion (exact Idle field values pinned during implementation).
- Because the timer is now `Idle` after switch, insert `start_timer_phase(task2)` before the tick-to-completion loop (lines 906-932) so the session-credit verification stays valid.

**Add new regression tests** (switch from each non-`Working` phase → assert `Active{new task, Idle}`):
- `should_switch_active_task_from_short_break_resets_to_idle`
- `should_switch_active_task_from_long_break_resets_to_idle`
- `should_switch_active_task_from_paused_resets_to_idle`
- `should_switch_active_task_to_same_task_is_noop` (idempotency — running timer is NOT stopped)

**Add tests for the strict path** (`switch_task`) similarly, at least one `Paused`-phase case, to lock Change 3.

## Out of Scope / Follow-ups

- **Redundant double-reset:** `progress_phase.rs` and `complete_flow.rs` call `switch_active_task` then `reset_timer_to_idle`. After Change 1 the second call is a harmless no-op (reset-on-Idle). Simplify as a follow-up.
- **Tray/non-calling-window sync** on switch (see Decision 2) — emit from the orchestrator after `load_state` if needed.
- **Old-task status on abandon** (product decision) — see Edge Cases.
- **Decoupling active-task from the timer** (see Decision 3) — backlog, not this fix.

## References

- `CLAUDE.md` — tick-loop ownership contract.
- `core/domain/src/timer/timer.rs:118-125` — `Timer::new` (the invariant constructor).
- `core/domain/src/timer/state_machine.rs:245` — `with_remaining_seconds` (the carry-state mechanism being removed).
- `core/usecases/src/task/switch_active_task.rs:55-71` — primary fix site.
- `core/usecases/src/task/switch_task.rs:86-90` — alignment fix site.
- `apps/tauri-app/src/commands/task_cmd/get_active_task.rs:15` — proof that `task_id` is the active-task source of truth.
- `apps/tauri-app/src/commands/timer_cmd/switch_active_task.rs` — orchestrator (no change needed).
- `core/infra/tests/app/adv_timer.rs:825` — existing test to update.
