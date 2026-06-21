# Module Map - Quick File Reference

## Domain Layer

### Timer Module
- `domain/src/timer/mod.rs` - Module exports
- `domain/src/timer/timer.rs` - Timer entity
- `domain/src/timer/state_machine.rs` - State transitions
- `domain/src/timer/service.rs` - Timer service trait
- `domain/src/timer/transitions.rs` - Transition logic
- `domain/src/timer/error.rs` - Timer errors
- `domain/src/timer/events/` - All timer events

### Task Module  
- `domain/src/task/mod.rs` - Module exports
- `domain/src/task/task.rs` - Task entity
- `domain/src/task/repository.rs` - Repository trait
- `domain/src/task/builder.rs` - Task builder
- `domain/src/task/status.rs` - Status enum
- `domain/src/task/id.rs` - TaskId type
- `domain/src/task/settings.rs` - Task settings
- `domain/src/task/cycling_srv.rs` - Cycling service
- `domain/src/task/events/` - All task events

### Config Module
- `domain/src/config/mod.rs` - Module exports
- `domain/src/config/config.rs` - Root config
- `domain/src/config/repo.rs` - Repository trait
- `domain/src/config/general.rs` - General settings
- `domain/src/config/audio.rs` - Audio settings
- `domain/src/config/appearance.rs` - UI settings
- `domain/src/config/notification.rs` - Notifications
- `domain/src/config/task_defaults.rs` - Default timers

### Audio Module
- `domain/src/audio/mod.rs` - Module exports
- `domain/src/audio/audio_srv.rs` - Service trait
- `domain/src/audio/asset.rs` - Audio asset
- `domain/src/audio/category.rs` - Categories
- `domain/src/audio/library.rs` - Asset library
- `domain/src/audio/error.rs` - Audio errors

### Shared Kernel
- `domain/src/shared_kernel/mod.rs` - Common types
- `domain/src/shared_kernel/events/` - Event system
- `domain/src/shared_kernel/value_objects/` - VOs
- `domain/src/shared_kernel/traits/` - Shared traits
- `domain/src/shared_kernel/errors.rs` - Base errors

## Use Cases Layer

### Timer Use Cases
- `usecases/src/timer/start_timer_session.rs`
- `usecases/src/timer/pause_timer_session.rs`
- `usecases/src/timer/reset_timer_session.rs`
- `usecases/src/timer/skip_timer_phase.rs`
- `usecases/src/timer/get_timer_state.rs`
- `usecases/src/timer/switch_timer_task.rs`

### Task Use Cases
- `usecases/src/task/create_task.rs`
- `usecases/src/task/update_task.rs`
- `usecases/src/task/delete_task.rs`
- `usecases/src/task/get_task.rs`
- `usecases/src/task/search_tasks.rs`
- `usecases/src/task/complete_session.rs`
- `usecases/src/task/cycle_task.rs`
- `usecases/src/task/switch_task.rs`
- `usecases/src/task/get_task_queue.rs`

### Config Use Cases
- `usecases/src/config/get_config.rs`
- `usecases/src/config/update_config.rs`
- `usecases/src/config/reset_config.rs`
- `usecases/src/config/import_config.rs`
- `usecases/src/config/export_config.rs`

### Audio Use Cases
- `usecases/src/audio/play_audio.rs`
- `usecases/src/audio/notification_audio.rs`
- `usecases/src/audio/manage_library.rs`

## Infrastructure Layer

### Main Application
- `infra/src/main.rs` - Tauri entry point
- `infra/src/bootstrap.rs` - DI container setup
- `infra/src/lib.rs` - Library exports
- `infra/src/commands/` - Tauri commands

### Adapters

#### Database Adapter
- `infra/src/adapters/database/mod.rs`
- `infra/src/adapters/database/connection.rs`
- `infra/src/adapters/database/models.rs`
- `infra/src/adapters/database/sqlite_task_repository.rs`
- `infra/src/adapters/database/sqlite_timer_repository.rs`
- `infra/src/adapters/database/sqlite_config_repository.rs`

#### Timer Adapter
- `infra/src/adapters/timer/mod.rs`
- `infra/src/adapters/timer/sqlite_service.rs`
- `infra/src/adapters/timer/repository.rs`
- `infra/src/adapters/timer/timer_dto.rs`
- `infra/src/adapters/timer/event_handlers/`

#### Task Adapter
- `infra/src/adapters/task/mod.rs`
- `infra/src/adapters/task/task_dto.rs`
- `infra/src/adapters/task/cycling_srv.rs`
- `infra/src/adapters/task/event_handlers/`

#### Audio Adapter
- `infra/src/adapters/audio/mod.rs`
- `infra/src/adapters/audio/audio_service_adapter.rs`
- `infra/src/adapters/audio/library_service.rs`
- `infra/src/adapters/audio/asset_provider.rs`

#### Config Adapter
- `infra/src/adapters/config/mod.rs`
- `infra/src/adapters/config/file_repo.rs`
- `infra/src/adapters/config/builder.rs`
- `infra/src/adapters/config/config_dto.rs`

#### Events Adapter
- `infra/src/adapters/events/mod.rs`
- `infra/src/adapters/events/mem_event_bus.rs`
- `infra/src/adapters/events/event_handler.rs`
- `infra/src/adapters/events/event_subscriber.rs`

### Commands (Tauri IPC)
- `infra/src/commands/timer_cmd.rs`
- `infra/src/commands/task_cmd.rs`
- `infra/src/commands/config_cmd.rs`
- `infra/src/commands/audio_cmd.rs`
- `infra/src/commands/notification_cmd.rs`

### Database
- `infra/src/schema.rs` - Diesel schema
- `infra/migrations/` - SQL migrations

### Tests
- `infra/tests/main.rs` - Test entry
- `infra/tests/timer/` - Timer tests
- `infra/tests/task/` - Task tests
- `infra/tests/audio/` - Audio tests
- `infra/tests/core/` - Test utilities

## UI Layer

### Main App
- `ui/src/lib.rs` - Library exports
- `ui/src/app.rs` - Root component

### Pages
- `ui/src/pages/timer/timer_page.rs`
- `ui/src/pages/timer/timer_vm.rs`
- `ui/src/pages/timer/timer_display.rs`
- `ui/src/pages/timer/timer_controls.rs`

- `ui/src/pages/task/task_page.rs`
- `ui/src/pages/task/task_vm.rs`
- `ui/src/pages/task/task_list.rs`
- `ui/src/pages/task/task_creation_form.rs`

- `ui/src/pages/settings/settings_page.rs`
- `ui/src/pages/settings/settings_vm.rs`
- `ui/src/pages/settings/settings_state.rs`

### Components
- `ui/src/components/navigation.rs`
- `ui/src/components/sidebar.rs`
- `ui/src/components/circular_progress.rs`
- `ui/src/components/task_cycle_controls.rs`
- `ui/src/components/error_toast.rs`

### Utils
- `ui/src/utils/events.rs` - Event handling
- `ui/src/utils/view_model.rs` - VM base

## Configuration Files

### Root Config
- `Cargo.toml` - Workspace config
- `justfile` - Build commands
- `rustfmt.toml` - Format config

### Infrastructure Config
- `infra/Cargo.toml` - Dependencies
- `infra/tauri.conf.json` - Tauri config
- `infra/diesel.toml` - Database config

### UI Config
- `ui/Cargo.toml` - Dependencies
- `ui/index.html` - Entry HTML
- `ui/styles.css` - Global styles

## Asset Files
- `assets/sounds/notifications/` - Alert sounds
- `assets/sounds/background/` - Ambient sounds

## Documentation
- `README.md` - Project readme
- `docs/` - All documentation
- `CLAUDE.md` - AI assistant context