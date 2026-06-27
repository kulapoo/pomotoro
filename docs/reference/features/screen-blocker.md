# Screen Blocker Feature

## Status

🟢 **ACTIVE** (re-implemented 2026-06-25)

A focus-enforcement feature that, when a work or break phase naturally expires,
displays a fullscreen overlay over the app window, optionally forcing the
window into fullscreen + always-on-top mode. The user dismisses it via ESC or a
button.

## Overview

Two distinct mechanisms are involved:
1. **Native-window block** — Tauri commands that set the window fullscreen and
   always-on-top (driven by the frontend on activate/deactivate).
2. **In-DOM overlay** — a React component (`ScreenBlocker.tsx`) rendering a
   blocking overlay independently of the native commands.

## Gating (global only)

The feature is **global**, controlled by four `GeneralConfig` toggles. There is
no per-task `enable_screen_blocking` flag — that was dead code in the original
implementation and was intentionally not re-introduced.

## Architecture (Data Flow)

```
[UC]   progress_phase()
   │  reads config.general.block_screen_after_work / block_screen_after_break
   │  (gated on cmd.from_phase: Work / ShortBreak|LongBreak)
   │  → returns PhaseOutcome::{Started,Paused} { block_message: Option<String> }
   ▼
[Infra] CountdownExpiredHandler.handle()
   │  if let Some(message) = block_message
   │      emitter.emit("screen_blocker:activate", { message })
   ▼
[Tauri event bus]  event "screen_blocker:activate"
   ▼
[TS]  useEventBus (EventBus.ts) listener → useScreenBlockerStore.activate(msg)
   │                                       + invoke("activate_screen_block")
   │                                                  ▼
   │                                       [Rust cmd] activate_screen_block
   │                                       (set_fullscreen(true) + set_always_on_top(true))
   ▼
[TS]  <ScreenBlocker /> (reads store) renders fullscreen overlay
        onDismiss (ESC/button) → store.dismiss()
          → invoke("deactivate_screen_block") (clears fullscreen + always-on-top)
          → clears isBlocking
```

## Config Fields (`GeneralConfig`)

| Field | Type | Default | Purpose |
|---|---|---|---|
| `block_screen_after_work` | bool | false | Gate the overlay after a Work phase expires |
| `block_screen_after_work_message` | String | "Work session complete. Time for a break." | Overlay message after Work |
| `block_screen_after_break` | bool | false | Gate the overlay after a Break phase expires |
| `block_screen_after_break_message` | String | "Break over. Back to work." | Overlay message after Break |

## Implementation Map

### Backend (Rust)

- **Event names** — `core/domain/src/event_names/ui_listeners.rs`:
  `screen_blocker::ACTIVATE = "screen_blocker:activate"`.
- **Config struct** — `core/domain/src/config/general.rs`: the 4 fields above
  with `#[serde(default)]`/`#[serde(default = "default_*_message")]` + the two
  `default_*_message()` fns + `Default` impl entries.
- **Decision logic** — `core/usecases/src/timer/progress_phase.rs`: computes
  `block_message: Option<String>` from `cmd.from_phase` + config, threads it
  into `PhaseOutcome::Started` / `::Paused` (NOT `Stopped`). Keeps the gating
  rule in the usecase layer; the infra handler stays dumb.
- **Emit** — `core/infra/src/adapters/timer/event_handlers/countdown_expired.rs`:
  in the `Started` / `Paused` arms, `if let Some(message) = block_message {
  emitter.emit(ACTIVATE, { message }) }`.
- **Tauri commands** — `apps/tauri-app/src/commands/screen_blocker_cmd/`:
  `activate_screen_block.rs` / `deactivate_screen_block.rs`, each injecting
  `AppHandle` + `Manager` and calling `set_fullscreen(...)` +
  `set_always_on_top(...)` on the `"main"` window.
- **Registration** — `apps/tauri-app/src/commands/mod.rs` (`pub mod` + `pub use`)
  and `apps/tauri-app/src/lib.rs` (`invoke_handler!`).
- **Test fixtures** — `core/infra/tests/core/fixtures/config_fixtures.rs`:
  `minimal_general()` includes the 4 new fields. Existing `task_cycling.rs`
  tests match on `PhaseOutcome` with `..`, so no edits were required.

### Frontend (TS/TSX)

- **Type-safe command/event maps** — `apps/react-ui/src/lib/tauri.ts`:
  - `commands`: `activateScreenBlock` / `deactivateScreenBlock`.
  - `CommandMap`: `activate_screen_block` / `deactivate_screen_block`.
  - `events`: `screenBlockerActivate`.
  - `EventPayloadMap`: `'screen_blocker:activate': { message: string }`.
- **Config type** — `apps/react-ui/src/pages/settings/useSettings.ts`:
  `GeneralConfig` has the 4 new fields.
- **Store** — `apps/react-ui/src/app/useScreenBlocker.ts`: zustand store with
  `isBlocking` / `message` / `activate(msg)` / `dismiss()`.
- **Event listener** — `apps/react-ui/src/app/EventBus.ts`: subscribes to
  `screen_blocker:activate` and calls into the store.
- **Component** — `apps/react-ui/src/components/ScreenBlocker.tsx`: fullscreen
  overlay (`position: fixed`, `z-index: 9999`, `role="dialog"`,
  `aria-modal="true"`), message + dismiss button, ESC handler, button focus.
- **App wiring** — `apps/react-ui/src/app/App.tsx`: renders `<ScreenBlocker />`
  as a sibling of `<main>`, inside `Bootstrap`.
- **Settings UI** — `apps/react-ui/src/pages/settings/components/GeneralTab.tsx`:
  a `<Section title="Screen Blocking">` with work/break toggles and conditional
  message text inputs.

## Notes

- **No per-task flag**: the original `enable_screen_blocking` plumbing through
  create/update task was not re-added; gating is purely global.
- **Deactivation is command-driven** (frontend), not event-driven. The backend
  never needs to dismiss the overlay.
- **Accessibility**: the overlay uses `role="dialog"` + `aria-modal="true"` and
  focuses the dismiss button on show.
