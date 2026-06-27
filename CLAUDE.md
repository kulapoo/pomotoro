# CLAUDE.md

- Must @agent-systems-architect when you need high-level system design and architectural decisions for Clean Architecture and Domain-Driven Design implementations

- Must use @agent-rust-developer when you need to write, refactor, or enhance Rust code following idiomatic patterns and project-specific style guidelines

## Tick-loop ownership

`TimerTickService::start_timer_tick_loop` / `stop_timer_tick_loop` / `load_state` MUST be called by the orchestrator that drives a state-changing `usecases::timer::*` call (a Tauri command, a tray handler, or `CountdownExpiredHandler`) — NOT by domain event handlers. Domain event handlers (`TimerStartedHandler`, `TimerResetHandler`, `TimerPausedHandler`, `TaskResetHandler`) are UI-only emitters; they never mutate `cancel_handle`.

When an orchestration needs both STOP and START:

```rust
timer_tick_service.stop_timer_tick_loop().await?;
timer_tick_service.load_state().await?;            // refresh in-memory cache
timer_tick_service.start_timer_tick_loop(cfg, None).await?;  // last-write-wins
```

Never `tokio::time::sleep` to "drain" an event handler. Never rely on event ordering. See `docs/superpowers/plans/2026-06-27-tick-loop-direct-drive.md` and `tmp/architect/27-06-2026-1332-tick-loop-boundary/design.md` for the rationale.
