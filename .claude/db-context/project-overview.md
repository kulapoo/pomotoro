# Pomotoro Project Overview

## Project Type
Pomodoro Timer Application with task management and productivity features

## Technology Stack
- **Backend**: Rust with Tauri framework
- **Frontend**: Leptos (Rust-based reactive UI)
- **Database**: SQLite with Diesel ORM
- **Architecture**: Clean Architecture (Domain-Driven Design)
- **Build**: Trunk for frontend, Cargo for Rust
- **Testing**: Rust native testing with integration/e2e tests

## Architecture Layers

### Domain Layer (`domain/`)
- Core business logic and entities
- Pure Rust, framework-agnostic
- Key modules: timer, task, config, audio, shared_kernel
- Event-driven with EventPublisher trait
- No external dependencies except std library

### Use Cases Layer (`usecases/`)
- Application-specific business rules
- Orchestrates domain entities
- Handles complex workflows
- Key categories: timer, task, config, audio operations

### Infrastructure Layer (`infra/`)
- Tauri application entry point
- External adapters (database, file system, notifications)
- Event bus implementation
- SQLite repositories
- Audio service implementation
- Configuration file management

### UI Layer (`ui/`)
- Leptos reactive components
- Pages: Timer, Tasks, Settings
- View models for state management
- Event handling with Tauri IPC

## Core Features

### Timer Management
- Work/Break session cycles (Pomodoro technique)
- Configurable session durations
- Pause, skip, reset functionality
- State machine for timer phases
- Visual progress indicators

### Task Management
- Create, update, delete tasks
- Task status tracking (pending, in_progress, completed)
- Session counting per task
- Task cycling strategies
- Search and filtering
- Default task assignment

### Audio System
- Background sounds (rain, ocean, cafe, etc.)
- Notification sounds for phase transitions
- Configurable volume and enabled states
- Audio asset management

### Configuration
- General settings (theme, auto-start)
- Audio preferences
- Notification settings
- Task defaults
- Import/export configuration

## Event System
- Domain events for decoupled communication
- Event handlers in infrastructure layer
- Key events:
  - Timer: Started, Paused, Reset, PhaseCompleted
  - Task: Created, Updated, StatusChanged, Completed
  - Session: WorkStarted, BreakStarted, Completed
  - App: Started, Exited

## Database Schema
- Tasks table: id, title, description, status, sessions_completed
- Timer state persistence
- Configuration storage
- SQLite with migrations in `infra/migrations/`

## Development Workflow
1. Domain modeling first
2. Use case implementation
3. Infrastructure adapters
4. UI components and integration
5. Testing at each layer

## Testing Strategy
- Unit tests in each module
- Integration tests in `infra/tests/`
- E2E tests for complete workflows
- Test utilities and builders for fixtures
- Mock implementations for testing

## Key Design Patterns
- Repository pattern for data access
- Service pattern for business logic
- Event-driven architecture
- State machine for timer
- Builder pattern for entities
- Strategy pattern for task cycling

## Current Focus Areas
- Migration from file storage to SQLite
- Timer service refactoring
- Event handler improvements
- Task cycling enhancements