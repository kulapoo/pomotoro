# Module Map — Quick File Reference

A tour of the codebase by location. The project is split into a
**framework-agnostic core** (`core/`) and **client apps** (`apps/`).

Dependency direction (inward): UI → Infrastructure → Use Cases → Domain.

## Repository Root

```
pomotoro/
├── core/                       # Framework-agnostic core engine
│   ├── domain/                 #   Business logic & entities
│   ├── usecases/               #   Application services (orchestrates domain)
│   └── infra/                  #   Infrastructure (repos, event bus, audio, timer tick)
├── apps/                       # Client applications (thin wrappers over core)
│   ├── tauri-app/              #   Tauri desktop client (commands, plugins, UI emission)
│   ├── react-ui/               #   React + TypeScript frontend (Vite)
│   ├── pomotoro-cli/           #   CLI client (planned)
│   └── cosmic-de/              #   Cosmic DE applet (planned)
├── assets/                     # Sounds (notifications/, background/)
├── scripts/                    # install-deps.sh, CI helpers
├── docs/                       # This documentation
├── Cargo.toml                  # Workspace root
├── justfile                    # All dev/build/test commands
├── rustfmt.toml
├── CLAUDE.md · GEMINI.md       # AI assistant context
└── README.md
```

## Domain Layer (`core/domain/src/`)

Pure business logic. No external dependencies, no I/O.

### Timer Module (`timer/`)
- `mod.rs` — module exports
- `timer.rs` — `Timer` entity
- `state_machine.rs` — state transitions
- `transitions.rs` — transition business rules
- `repository.rs` — `TimerRepository` trait
- `id.rs` — `TimerId`
- `error.rs` — timer errors
- `README.md` — in-source notes

### Task Module (`task/`)
- `mod.rs` — module exports
- `task.rs` — `Task` entity
- `builder.rs` — `TaskBuilder`
- `repository.rs` — `TaskRepository` trait
- `cycle_service.rs` — task cycling logic
- `status.rs` — `TaskStatus` enum
- `id.rs` — `TaskId`
- `test_builder.rs` — test fixture builder

### Config Module (`config/`)
- `mod.rs` — module exports
- `config.rs` — root `Config`
- `repo.rs` — `ConfigRepository` trait
- `general.rs` — general settings (theme, auto-start, screen blocking, tray)
- `audio.rs` — audio settings
- `appearance.rs` — UI appearance settings
- `notification.rs` — notification settings

### Audio Module (`audio/`)
- `mod.rs` — module exports
- `audio_srv.rs` — `AudioService` trait
- `asset.rs` — `AudioAsset`
- `category.rs` — `AudioCategory`
- `library.rs` — `AudioLibrary`
- `error.rs` — audio errors

### Shared Kernel (`shared_kernel/`)
- `mod.rs` — common types
- `errors.rs` — base error types
- `serde_utils.rs` — serialization helpers

### Event Names (`event_names/`)
Central registry of emit/listen string constants shared with the frontend.
- `mod.rs`
- `commands.rs` — Tauri command names
- `ui_listeners.rs` — event names emitted to the UI (e.g. `screen_blocker:activate`)

## Use Cases Layer (`core/usecases/src/`)

Application services that orchestrate the domain.

### Timer Use Cases (`timer/`)
`start_timer_phase`, `resume_timer_phase`, `pause_timer_phase`,
`reset_timer_phase`, `reset_timer_to_idle`, `skip_timer_phase`,
`complete_timer_phase`, `progress_phase`, `update_timer_secs`,
`clear_active_task`.

### Task Use Cases (`task/`)
`create_task`, `update_task`, `update_task_settings`, `reset_task_settings`,
`delete_task`, `reset_task`, `reset_tasks`, `get_task`, `search_tasks`,
`switch_task`, `switch_active_task`, `complete_task`.

### Config Use Cases (`config/`)
`get_config`, `update_config`, `reset_config`, `export_config`,
`import_config`.

### Audio Use Cases (`audio/`)
`play_audio`, `notification_audio`, `manage_library`.

### Wiring
- `bootstrap.rs` — use-case construction / DI
- `lib.rs` — crate exports

## Infrastructure Layer (`core/infra/src/`)

SQLite persistence (Diesel + r2d2), the in-memory event bus, adapters, and
event handlers. **Zero Tauri dependencies** — reusable by any client.

### Adapters (`adapters/`)
- `database/` — `connection.rs` (r2d2 pool), `models.rs`, `sqlite_config_repository.rs`
- `timer/` — `sqlite_repository.rs`, `sqlite_service.rs`, `timer_dto.rs` + `event_handlers/`
- `task/` — `sqlite_repository.rs` + `event_handlers/` (created, updated, deleted, status_changed, completed, reset, active_changed, registry)
- `config/` — adapter + `event_handlers/` (config_updated, config_reset, registry)
- `audio/` — `audio_service_adapter.rs`, `library_service.rs`, `asset_provider.rs`, `audio_service_wrapper.rs`, `event_handlers.rs`
- `notifications/` — `service.rs` + `event_handlers.rs`
- `events/` — `mem_event_bus.rs`, `event_handler.rs`, `event_subscriber.rs`, `app_emitter.rs`, `logging_emitter.rs`, `app_started_handler.rs`, `audio_events.rs`

### Top-level
- `bootstrap.rs` — `AppState` wiring / dependency injection
- `schema.rs` — Diesel schema (generated via `diesel print-schema`)
- `bin/test_db.rs` — DB inspection binary
- `lib.rs` — crate exports

### Tests (`core/infra/tests/`)
- `app/` — application-level integration tests
- `core/` — shared test infrastructure
  - `context/` — test context builders
  - `database/` — test database utilities
  - `fixtures/` — test data fixtures (e.g. `config_fixtures.rs`)
  - `mocks/` — mock implementations

## Apps

### Tauri Desktop Client (`apps/tauri-app/src/`)
Thin shell over `core/infra`. Owns window, tray, and Tauri command registration.
- `main.rs` / `lib.rs` — entry point + `invoke_handler!` registration
- `commands/` — `#[tauri::command]` handlers (timer, task, config, audio, notification, screen blocker)
- `adapters/` — `emitter.rs` (Tauri event emission), `notification_service.rs`
- `tray.rs` — system tray + tray-icon countdown
- `capabilities/`, `icons/`, `gen/` — Tauri config assets

### React Frontend (`apps/react-ui/src/`)
Feature-sliced: each feature owns its `types.ts`, `model/` (Zustand store),
`components/`, and `pages/`. Cross-feature imports go through `@/lib/`.
- `app/` — root `App.tsx`, `EventBus.ts`, app-wide stores (`useScreenBlocker`, …)
- `components/` — `layout/` + `ui/` (Row, Section, Toggle, NumberInput, SelectInput, …)
- `lib/` — `tauri.ts` (typed `invokeCmd` bridge + command/event maps), `logger`, `errors`, duration helpers
- `pages/` — `timer/`, `tasks/`, `settings/`

### Planned Clients
- `apps/pomotoro-cli/` — CLI client
- `apps/cosmic-de/` — Cosmic desktop applet

## Build & Config Files
- `Cargo.toml` / `Cargo.lock` — workspace + per-crate manifests (`core/{domain,usecases,infra}`, `apps/tauri-app`)
- `justfile` — canonical command runner (`just dev`, `just test`, `just ci`, …)
- `rustfmt.toml` — formatting config
- `apps/tauri-app/tauri.conf.json` — Tauri bundler config
- `core/infra/diesel.toml` — Diesel ORM config
- `apps/react-ui/{package.json, vite.config, tsconfig}` — frontend toolchain
- `assets/sounds/{notifications,background}/` — audio assets

## Documentation
- `README.md` — project README
- `docs/` — this documentation tree (see `docs/README.md` for the index)
- `CLAUDE.md` / `GEMINI.md` — AI assistant context

## Files Often Modified Together
- **Timer changes**: `core/domain/src/timer/` · `core/usecases/src/timer/` · `core/infra/src/adapters/timer/`
- **Task changes**: `core/domain/src/task/` · `core/usecases/src/task/` · `core/infra/src/adapters/task/`
- **Event changes**: `core/domain/src/event_names/` · `core/infra/src/adapters/events/` · `apps/react-ui/src/app/EventBus.ts`
- **A new Tauri command**: `apps/tauri-app/src/commands/` + registration in `lib.rs` + the matching entry in `apps/react-ui/src/lib/tauri.ts`

## Common Debugging Locations
- DI wiring: `core/infra/src/bootstrap.rs`
- Event bus: `core/infra/src/adapters/events/mem_event_bus.rs`
- DB connection pool: `core/infra/src/adapters/database/connection.rs`
