# Proposal — Further `sqlite_service.rs` Simplification

**Status:** Proposal (awaiting approval). Not yet scheduled.

**Context:** Follow-up to the Tier-1 refactor executed on 2026-06-28
(`2026-06-28-rust-backend-refactor.md`), which flattened the tick-loop pyramid
of doom and reduced `start_timer_tick_loop` from 111 to ~25 lines. This doc
assesses whether the file warrants more work now that it is already readable.

**TL;DR — recommendation:** Yes, but only the two low-risk extractions in
**Scope A** (≈30 lines removed, zero behavior change, single file). Defer the
structural split (Scope B) — its cost now outweighs the benefit since the file
is no longer a hotbed of nesting.

---

## Current state (post Tier-1 refactor)

`core/infra/src/adapters/timer/sqlite_service.rs` (363 lines):
- `start_timer_tick_loop` — ~25 clean lines: `load_state` → `resolve_timer_config`
  → `abort_existing_loop` → `tokio::spawn(run_tick_loop(...))` → store handle.
- `run_tick_loop` + `compute_tick_outcome` — free functions, single-lock tick.
- State cache methods: `save_state`, `load_state`, `get_current_timer`,
  `with_timer`, `update_timer`, `reset_timer`, `reset_timer_phase`.

The nesting problem is gone. What remains is **duplication**, not complexity.

---

## Scope A — Recommended (low risk, single file, no behavior change)

### A1. Extract `mutate_and_persist<F>`

`update_timer`, `reset_timer`, and `reset_timer_phase` (lines 211–270) all
repeat the identical pattern:

```rust
{
    let mut timer = self.timer.lock().await;
    <mutate timer, return DomainResult>
}
self.save_state().await
```

Extract:

```rust
async fn mutate_and_persist<F>(&self, f: F) -> DomainResult<()>
where
    F: FnOnce(&mut Timer) -> DomainResult<()>,
{
    {
        let mut timer = self.timer.lock().await;
        f(&mut timer)?;
    }
    self.save_state().await
}
```

The three callers collapse to one-liners, removing ~20 lines and the only
remaining structural duplication in the impl block.

**Risk:** None — pure mechanical extraction behind a private helper. The
"drop the guard before the repo write" invariant (documented on `save_state`)
is preserved verbatim.

### A2. Collapse `TickOutcome` stop-literals

`compute_tick_outcome` (lines 285–330) constructs the same all-false
`TickOutcome` literal three times (not-running, no active task, tick error).
Add a `stopped()` constructor:

```rust
impl TickOutcome {
    fn stopped() -> Self {
        Self {
            should_continue: false,
            phase_completed: false,
            events_to_publish: Vec::new(),
            expiry_payload: None,
        }
    }
}
```

The three early-returns become `return TickOutcome::stopped();`. Removes
~14 lines of repeated literals.

**Risk:** None.

### Scope A verification

`just test-infra && just clippy` — the existing `tick_loop_invariants.rs` and
`concurrent_command_stress.rs` are the canaries; both already exercise the tick
loop and the reset paths.

---

## Scope B — Deferred (recommend NOT doing now)

### B1. Split `TimerTickService` into `TimerStateCache` + `TickLoopController` + facade

This was the Tier-2 item explicitly deferred by the original refactor plan.
It would make the "state cache vs loop lifecycle vs serialization" boundary
mechanically enforceable.

**Why defer now:** the original motivation was unreadable nesting. That is
fixed. The split now touches `CountdownExpiredHandler` internals and the
`AppContext` test wiring for a marginal architectural-purity gain. Revisit if
a second loop type is introduced or if a third concern accretes onto the
service.

### B2. Unify `start_timer_tick_loop`'s `Result<(), String>` → `DomainResult<()>`

The lone `String`-returning method forces every caller to
`.map_err(|e| Error::RepositoryError { message: e })`. Unifying to
`DomainResult<()>` is cleaner but fans out to ~10 call sites across
`apps/tauri-app` and tests. Worth doing eventually as part of a broader
error-type consistency pass, not as a standalone change.

### B3. Repository-error mapping helper

The `.map_err(|e| Error::RepositoryError { message: e.to_string() })` idiom
repeats in `save_state`/`load_state`. A `From` impl or a private
`repo_err(e)` fn would tidy this up. Trivial; bundle with B2 if attempted.

---

## Decision needed

- **Approve Scope A only** (recommended) → ~30 lines removed, single file,
  zero behavior change, one short PR.
- **Approve Scope A + B2** → adds the error-type unification (touches callers).
- **Defer everything** → the file is good enough after Tier 1; close this out.
