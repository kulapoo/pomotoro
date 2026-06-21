# Screen Blocker Feature

## Status

🔴 **DISABLED / REMOVED** (2026-06-21)

The entire ScreenBlocker feature was removed across both the Rust backend and
the React frontend. This document captures the original architecture so the
feature can be re-implemented later if desired.

## Overview

A focus-enforcement feature that, when a work or break phase naturally expires,
displays a fullscreen overlay over the app window, optionally forcing the
window into fullscreen + always-on-top mode. The user dismisses it via ESC or a
button.

Two distinct mechanisms were involved:
1. **Native-window block** — Tauri commands that set the window fullscreen and
   always-on-top (driven by the frontend on activate/deactivate).
2. **In-DOM overlay** — a React component (`ScreenBlocker.tsx`) rendering a
   blocking overlay independently of the native commands.

## Why It Was Removed

- The feature was off by default (all gating config fields defaulted to `false`).
- The `enable_screen_blocking` per-task flag was plumbed through the
  create/update task flows but **never read** by any runtime gating logic —
  i.e. dead code.
- Removed to simplify the config surface and reduce the number of unused code
  paths. This doc preserves the wiring map for a future, cleaner re-implementation.

## Original Architecture (Data Flow)

```
[Rust] CountdownExpiredHandler.handle()
   │  reads config.general.block_screen_after_work / block_screen_after_break
   │  (if should_block) → emitter.emit("screen_blocker:activate", { message })
   ▼
[Tauri event bus]  event "screen_blocker:activate"
   ▼
[TS]  App.tsx listens → setBlockingMessage + setIsBlocking(true)
   │                        + invoke("activate_screen_block")
   │                                              ▼
   │                        [Rust cmd] activate_screen_block
   │                        (set_fullscreen(true) + set_always_on_top(true))
   ▼
[TS]  <ScreenBlocker message onDismiss> renders fullscreen overlay
         onDismiss → setIsBlocking(false) + invoke("deactivate_screen_block")
```

## Original Config Fields (`GeneralConfig`)

| Field | Type | Default | Purpose |
|---|---|---|---|
| `enable_screen_logging` | bool | false | Per-task flag (was **orphaned** — never read by runtime) |
| `block_screen_after_work` | bool | false | Gate the overlay after a Work phase expires |
| `block_screen_after_work_message` | String | "Work session complete. Time for a break." | Overlay message after Work |
| `block_screen_after_break` | bool | false | Gate the overlay after a Break phase expires |
| `block_screen_after_break_message` | String | "Break over. Back to work." | Overlay message after Break |

> ⚠️ **Note for re-implementation:** the decision logic in
> `CountdownExpiredHandler` only consulted `block_screen_after_work` /
> `block_screen_after_break`. If re-introducing `enable_screen_blocking`,
> wire it into the gating `match` so it is no longer dead code.

## Re-implementation Checklist

### Backend (Rust)

1. **Event names**
   - `core/domain/src/event_names/ui_listeners.rs` — re-add:
     ```rust
     pub mod screen_blocker {
         pub const ACTIVATE: &str = "screen_blocker:activate";
     }
     ```
   - `core/domain/src/event_names/commands.rs` — re-add (optional, currently
     the frontend invokes string literals directly):
     ```rust
     pub mod screen_blocker {
         pub const ACTIVATE: &str = "activate_screen_block";
         pub const DEACTIVATE: &str = "deactivate_screen_block";
     }
     ```

2. **Config struct**
   - `core/domain/src/config/general.rs` — re-add the 5 fields above with
     `#[serde(default ...)]` attributes and the two `default_*_message()` fns,
     plus their entries in the `Default` impl.

3. **Decision logic**
   - `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs` —
     after `let config = self.config_repository.get_config().await?;`,
     re-add the `should_block` match and the
     `self.emitter.emit(ui_listeners::screen_blocker::ACTIVATE, json!({ "message": message }))`
     block.

4. **Tauri commands**
   - Recreate `apps/tauri-app/src/commands/screen_blocker_cmd.rs` with
     `activate_screen_block` / `deactivate_screen_block` using
     `window.set_fullscreen(...)` + `window.set_always_on_top(...)`.
   - Register the module in `apps/tauri-app/src/commands/mod.rs`
     (`pub mod` + `pub use`).
   - Register the commands in the `invoke_handler` in `apps/tauri-app/src/lib.rs`.

5. **Per-task plumbing (only if re-introducing `enable_screen_blocking`)**
   - `apps/tauri-app/src/commands/task_cmd/create_task.rs` — field on
     `CreateTaskRequest` + application to `default_config.general`.
   - `apps/tauri-app/src/commands/task_cmd/update_task.rs` — field on
     `UpdateTaskRequest` + passthrough to `UpdateTaskCmd`.
   - `core/usecases/src/task/update_task.rs` — field on `UpdateTaskCmd` +
     application to `task.config_mut().general`.

6. **Test fixtures**
   - `core/infra/tests/core/fixtures/config_fixtures.rs` — add the fields back
     to `minimal_general()`.

### Frontend (TS/TSX)

7. **Component**
   - Recreate `apps/react-ui/src/components/ScreenBlocker.tsx` — a fullscreen
     overlay (`position: fixed`, `z-index: 9999`) with message + dismiss
     button, ESC-to-dismiss key handler.

8. **Types**
   - `apps/react-ui/src/types/index.ts` — re-add the 5 fields to
     `GeneralConfig` and `ScreenBlockerActivate: 'screen_blocker:activate'`
     to `AppEvents`.

9. **App wiring**
   - `apps/react-ui/src/App.tsx`:
     - import `ScreenBlocker` and `invoke` from `@tauri-apps/api/core`
     - add `isBlocking` / `blockingMessage` state
     - add `handleDismissBlocker` callback (`useCallback`)
     - add the `listen(AppEvents.ScreenBlockerActivate, ...)` listener and
       its cleanup
     - render `<ScreenBlocker ... />` when `isBlocking`

10. **Settings UI**
    - `apps/react-ui/src/pages/SettingsPage.tsx` — re-add a
      `<Section title="Screen Blocking">` in the `GeneralTab` with toggles for
      `block_screen_after_work` / `block_screen_after_break` and conditional
      message text inputs.

## Design Notes for a Cleaner Re-implementation

- **Gate on `enable_screen_blocking` too**: if re-adding the per-task flag,
  fold it into the `should_block` decision so it actually does something.
- **Consider a `DEACTIVATE` event**: the original code had a commented-out
  `ui_listeners::screen_blocker::DEACTIVATE` emit with no corresponding
  constant. Decide whether deactivation should be event-driven (backend) or
  command-driven (frontend, as it was).
- **Accessibility**: the overlay used `role="dialog"` + `aria-modal="true"`;
  keep this and ensure focus trapping if re-implementing.
