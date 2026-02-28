# Decouple Infrastructure from Tauri

## Context

The `infra/` crate currently bundles **reusable infrastructure** (repositories, event bus, audio, timer tick service) with **Tauri-specific glue** (commands, plugins, state management, UI event emission). This prevents other desktop clients — such as a Pop!_OS Cosmic applet — from reusing our core pomodoro capabilities without pulling in the entire Tauri dependency tree.

## Goal

Allow any Rust-based client to depend on a `pomotoro-core` (or similar) crate that provides the full pomodoro engine (persistence, events, audio, timer ticking) without any Tauri dependency. The Tauri desktop app becomes just one thin client on top of that core.

## Current Tauri Coupling Points

| Location | Tauri Dependency | Purpose |
|---|---|---|
| `core/infra/src/lib.rs` | `tauri::Builder`, `tauri::Manager`, `tauri::Emitter` | App setup, state management, command registration |
| `core/infra/src/bootstrap.rs` | `tauri::AppHandle` | Passed to `register_handlers` for emitter + notifications |
| `core/infra/src/commands/` (43 commands) | `#[tauri::command]`, `State<'_, T>` | Frontend-backend bridge |
| `core/infra/src/adapters/events/app_emitter.rs` | `tauri::AppHandle`, `tauri::Emitter` | `TauriAppHandleEmitter` implements `Emitter` trait |
| `core/infra/src/adapters/notifications/service.rs` | `tauri::AppHandle`, `tauri_plugin_notification` | Desktop notifications via Tauri plugin |
| `core/infra/src/adapters/notifications/event_handlers.rs` | `tauri::AppHandle` | Constructs `NotificationService` |
| `core/infra/Cargo.toml` | `tauri`, `tauri-plugin-*`, `tauri-build` | Direct dependencies |

## Architecture After Decoupling

```
┌──────────────────────────────────────────────┐
│  Clients (each their own crate)              │
│  ┌─────────────┐  ┌──────────────────────┐   │
│  │apps/tauri-  │  │ apps/cosmic-de/      │   │
│  │  app/       │  │ (Cosmic DE widgets,  │   │
│  │ (Tauri cmds,│  │  libcosmic UI,       │   │
│  │  plugins,   │  │  D-Bus notifs)       │   │
│  │  UI emit)   │  │                      │   │
│  └──────┬──────┘  └──────────┬───────────┘   │
│         │                    │               │
│  ┌──────▼────────────────────▼───────────┐   │
│  │  core/infra/  (pomotoro-core)         │   │
│  │  - Repositories (SQLite)              │   │
│  │  - Event Bus (InMemoryEventBus)       │   │
│  │  - Timer Tick Service                 │   │
│  │  - Audio Service (Rodio)              │   │
│  │  - Event Handlers (domain reactions)  │   │
│  │  - Bootstrap (AppRegistry)            │   │
│  │  - Emitter trait (abstract)           │   │
│  │  - NotificationServiceTrait (abstract)│   │
│  └──────────────────┬───────────────────┘    │
│  ┌──────────────────▼───────────────────┐    │
│  │  core/usecases/                       │   │
│  └──────────────────┬───────────────────┘    │
│  ┌──────────────────▼───────────────────┐    │
│  │  core/domain/                         │   │
│  └──────────────────────────────────────┘    │
└──────────────────────────────────────────────┘
```

## Implementation Plan

### Step 1: Extract Tauri commands into a new `apps/tauri-app/` crate

Create a new workspace member `apps/tauri-app/` that owns everything Tauri-specific:

**Moves from `core/infra/` → `apps/tauri-app/`:**
- `src/lib.rs` (the `run()` function with `tauri::Builder`)
- `src/commands/` (all 43 command handlers)
- `src/adapters/events/app_emitter.rs` → `TauriAppHandleEmitter` impl
- `src/adapters/notifications/service.rs` → `NotificationService` (Tauri-based impl)
- `src/adapters/notifications/event_handlers.rs` → notification handler registration
- `tauri.conf.json`, `build.rs`, `capabilities/`, `icons/`, `gen/`, `.taurignore`

**`apps/tauri-app/Cargo.toml` dependencies:**
- `infra` (path dependency — the core engine: `path = "../../core/infra"`)
- `domain`, `usecases` (paths: `../../core/domain`, `../../core/usecases`)
- `tauri`, all `tauri-plugin-*` crates
- `serde`, `serde_json`

### Step 2: Make `core/infra/` Tauri-free

**Remove from `core/infra/`:**
- All `tauri` and `tauri-plugin-*` dependencies from `Cargo.toml`
- Remove `tauri-build` from build-dependencies
- Remove `build.rs` (Tauri build script)
- Remove `tauri.conf.json`, `capabilities/`, `icons/`, `gen/`

**Refactor `bootstrap.rs`:**
- Remove `AppHandle` parameter from `bootstrap()` and `register_handlers()`
- `register_handlers` takes `Arc<dyn Emitter>` and `Arc<dyn NotificationServiceTrait>` instead of `AppHandle`
- Clients provide their own `Emitter` and `NotificationServiceTrait` implementations

```rust
// core/infra/src/bootstrap.rs — AFTER
pub async fn bootstrap(
    emitter: Arc<dyn Emitter>,
    notification_service: Arc<dyn NotificationServiceTrait>,
) -> Result<AppRegistry> { ... }
```

**Keep the `Emitter` trait in `core/infra/`** (it's already abstract):
```rust
// core/infra/src/adapters/events/app_emitter.rs — keep only the trait
pub trait Emitter: Send + Sync {
    fn emit(&self, event: &str, payload: Value) -> anyhow::Result<()>;
}
```

**Keep `NotificationServiceTrait` in `core/infra/`** (already abstract):
- Move `NotificationContext`, `NotificationEvent`, and the trait to stay in `core/infra/`
- Delete only the `NotificationService` struct (Tauri impl) — that moves to `apps/tauri-app/`

### Step 3: Update notification handler registration

Currently `register_notification_handlers` takes `AppHandle` and constructs `NotificationService` internally. Refactor to accept `Arc<dyn NotificationServiceTrait>` instead:

```rust
// core/infra — AFTER
pub async fn register_notification_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    notification_service: Arc<dyn NotificationServiceTrait>,
    config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
) -> Result<()> { ... }
```

### Step 4: Update workspace configuration

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "core/domain",
    "core/usecases",
    "core/infra",
    "apps/tauri-app",
    "apps/leptos-ui",
]
```

**Rename `infra` package** (optional but recommended):
- `name = "pomotoro-core"` in `core/infra/Cargo.toml`
- Or keep `infra` name — the directory name communicates the layer

### Step 5: Wire up `apps/tauri-app/`

The new `apps/tauri-app/src/lib.rs` does what `core/infra/src/lib.rs` does today:

```rust
// apps/tauri-app/src/lib.rs
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let emitter = Arc::new(TauriAppHandleEmitter::new(app_handle.clone()));
            let notif_service = Arc::new(NotificationService::new(app_handle, ...));

            let registry = block_on(infra::bootstrap(emitter, notif_service))?;

            // Register Tauri state...
            // Emit app:initialized...
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Commands in `apps/tauri-app/src/commands/` are thin wrappers that extract `State<T>` and delegate to `usecases`.

## Files Modified

| File | Action |
|---|---|
| `Cargo.toml` (workspace) | Add `apps/tauri-app` member |
| `core/infra/Cargo.toml` | Remove all `tauri*` deps |
| `core/infra/src/lib.rs` | Remove `run()`, keep `pub mod adapters` |
| `core/infra/src/bootstrap.rs` | Replace `AppHandle` with trait objects |
| `core/infra/src/adapters/events/app_emitter.rs` | Keep trait only, remove `TauriAppHandleEmitter` |
| `core/infra/src/adapters/notifications/service.rs` | Keep trait + enums, remove Tauri impl |
| `core/infra/src/adapters/notifications/mod.rs` | Update registration signature |
| `core/infra/src/commands/` | **Move entirely** to `apps/tauri-app/` |
| `apps/tauri-app/` (new) | New crate with all Tauri-specific code |

## Test Migration Strategy

### Current Test Inventory (~60+ tests)

The existing tests in `infra/tests/` fall into two categories:

**Pure infrastructure tests (stay in `core/infra/`):** ~25 tests
- `InMemoryEventBus` unit tests (11 tests in `src/adapters/events/mem_event_bus.rs`)
- `TestDatabase` tests (3 tests in `tests/core/database/test_database.rs`)
- `AppContext` unit tests (3 tests in `tests/core/context/app_context.rs`)
- `AppContextBuilder` unit tests (8 tests in `tests/core/context/builder.rs`)
- `MockAudioService` unit tests (6 tests in `tests/core/mocks/audio_service.rs`)
- Config/task fixture tests (2 tests)

**Integration tests using MockAppHandle (stay in `core/infra/`):** ~35 tests
- `tests/app/setup.rs` — bootstrap & initialization
- `tests/app/task.rs` — task CRUD + event flow (5 tests)
- `tests/app/timer.rs` — timer lifecycle (14 tests)
- `tests/app/config.rs` — config management (15+ tests)
- `tests/app/adv_timer.rs` — advanced pomodoro cycles

### Key Insight: Tests Already Use Mock Emitter

The test infrastructure already decouples from Tauri:
- `MockAppHandle` in `tests/core/mocks/ui/app_handle.rs` implements the **`Emitter` trait** (not Tauri's AppHandle)
- `UiSimulator` uses `MockAppHandle` — no Tauri types involved
- `AppContext` injects `MockAppHandle` as the emitter

This means **all ~60 tests stay in `core/infra/` unchanged** after decoupling. They never depended on real Tauri types — they only depend on the `Emitter` trait which remains in `core/infra/`.

### What Changes for Tests

1. **`AppContext` bootstrap path** — currently calls `infra::bootstrap(app_handle)` with a mock. After refactor, it calls `infra::bootstrap(emitter, notification_service)` with mock implementations. Update the 2 lines in `app_context.rs` that construct the context.

2. **Notification tests** — if any integration tests exercise notification handlers, they need a `MockNotificationService` implementing `NotificationServiceTrait`. Check if one exists; if not, create a simple no-op mock. The trait is already defined so this is trivial.

3. **No test files move to `apps/tauri-app/`** — the current tests don't test Tauri commands directly (they test through repositories and event bus). If command-level tests are needed later, they'd go in `apps/tauri-app/tests/`.

### Regression Checklist

Before and after the refactor, run:

```bash
# All infra tests must pass (core engine)
cargo test -p infra

# All domain tests must pass (unchanged)
cargo test -p domain

# All usecase tests must pass (unchanged)
cargo test -p usecases

# Tauri app builds and runs (manual verification)
cargo build -p tauri-app
cargo tauri dev  # (from tauri-app/ directory)
```

### Specific Regressions to Watch For

| Risk | Mitigation |
|---|---|
| Event handlers not registered after bootstrap refactor | Integration tests in `tests/app/` verify full event flow (timer start → tick → phase complete → break) |
| Notification handlers broken by signature change | `register_notification_handlers` tests in `tests/app/setup.rs` verify handler registration |
| Timer tick service doesn't start | `tests/app/timer.rs` has 14 tests covering start/pause/resume/reset/skip |
| Config persistence broken | `tests/app/config.rs` has 15+ tests covering save/load/reset/update |
| Audio events not firing | `MockAudioService` tests verify audio handler registration |
| `Emitter` trait not wired correctly | `UiSimulator` tests verify events reach the mock UI |

## Verification

1. `cargo build -p infra` — compiles with **zero** Tauri dependencies
2. `cargo build -p tauri-app` — compiles and runs the desktop app as before
3. `cargo test -p infra` — all ~60 existing tests pass unchanged
4. `cargo test -p domain && cargo test -p usecases` — domain and usecase tests unaffected
5. `cd apps/tauri-app && cargo tauri dev` — app launches and works identically from user perspective
6. Verify dependency tree: `cargo tree -p infra | grep -i tauri` returns nothing
