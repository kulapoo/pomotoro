# Pomotoro Application - Object Relationships

```mermaid
classDiagram
    %% Core Domain Entities
    class Task {
        +TaskId id
        +String name
        +u8 max_sessions
        +u8 current_sessions
        +TaskStatus status
        +is_completed() bool
        +increment_session() void
    }
    
    class Timer {
        +TimerStatus status
        +Phase phase
        +u32 remaining_seconds
        +tick() bool
        +format_time() String
    }
    
    class TimerState {
        +Timer timer
        +TaskId active_task_id
        +TimerConfiguration config
        +switch_task() void
        +next_phase() void
    }
    
    class Config {
        +TaskDefaults defaults
        +AudioConfig audio
        +validate() void
    }
    
    %% Enums
    class TaskStatus {
        <<enumeration>>
        Active
        Queued
        Paused
        Completed
    }
    
    class TimerStatus {
        <<enumeration>>
        Stopped
        Running
        Paused
    }
    
    class Phase {
        <<enumeration>>
        Work
        ShortBreak
        LongBreak
    }
    
    %% Repository Interfaces
    class TaskRepository {
        <<interface>>
        +save(task) void
        +find_by_id(id) Task
        +find_all() Task[]
    }
    
    %% Application Use Cases
    class CreateTask {
        +execute(repo, cmd) TaskId
    }
    
    class StartSession {
        +execute(timer_srv) void
    }
    
    %% Infrastructure
    class FileTaskRepository {
        +save(task) void
        +find_by_id(id) Task
    }
    
    class TimerService {
        +start() void
        +pause() void
        +get_state() TimerState
    }
    
    %% Controllers
    class TaskController {
        +create_task() TaskId
        +get_tasks() Task[]
    }
    
    class TimerController {
        +start_session() void
        +get_timer_state() TimerState
    }
    
    %% Basic Relationships - Using simple notation
    TimerState --> Timer
    TimerState --> TaskId
    Task --> TaskStatus
    Timer --> TimerStatus
    Timer --> Phase
    FileTaskRepository ..|> TaskRepository
    CreateTask --> TaskRepository
    StartSession --> TimerService
    TaskController --> CreateTask
    TimerController --> StartSession
```

## Architecture Overview

### Domain Layer (Pure Business Logic)
- **Task**: Core entity managing pomodoro sessions
- **Timer**: Countdown logic and phase management
- **TimerState**: Orchestrates timer with active task
- **Config**: Global application settings

### Application Layer (Use Cases)
- **CreateTask**: Creates new tasks with validation
- **StartSession**: Initiates timer sessions
- **UpdateTask**: Modifies existing tasks
- **SwitchTask**: Changes active task

### Infrastructure Layer (Technical Implementation)
- **FileTaskRepository**: File-based task persistence
- **TimerService**: Timer state management
- **AudioService**: Sound and notification handling
- **NotificationService**: System notifications

### Presentation Layer (External Interface)
- **TaskController**: Task-related API endpoints
- **TimerController**: Timer control endpoints
- **AudioController**: Audio management endpoints

## Key Object Interactions

1. **Task Management Flow**:
   - UI → TaskController → CreateTask → TaskRepository → File System

2. **Timer Session Flow**:
   - UI → TimerController → StartSession → TimerService → TimerState → Timer

3. **Task Switching Flow**:
   - UI → TaskController → SwitchTask → TimerService + TaskRepository

4. **Configuration Flow**:
   - UI → ConfigController → ConfigRepository → File System

5. **Event Flow**:
   - Domain Entities → Events → Infrastructure Services → External Systems

## Design Principles

- **Clean Architecture**: Dependencies point inward toward domain
- **Domain-Driven Design**: Rich domain models with business logic
- **Dependency Inversion**: Interfaces in domain, implementations in infrastructure
- **Single Responsibility**: Each class has one reason to change
- **Event-Driven**: Loose coupling through domain events