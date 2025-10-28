# Domain Layer Architecture

## Overview

The domain layer implements the core business logic for the Pomodoro timer application following Domain-Driven Design (DDD) principles. It contains all business entities, value objects, domain services, and events.

## Module Structure

```mermaid
graph TB
    subgraph "Domain Layer"
        SK[Shared Kernel]
        T[Task Module]
        TM[Timer Module]
        C[Config Module]
        A[Audio Module]
        E[Events Module]
        
        SK --> T
        SK --> TM
        SK --> C
        SK --> A
        E --> T
        E --> TM
    end
```

## Core Modules

### 1. Shared Kernel
Common domain primitives and interfaces used across all modules.

```mermaid
classDiagram
    class SharedKernel {
        <<module>>
    }
    
    class Event {
        <<trait>>
        +id() Uuid
        +timestamp() DateTime
        +event_type() String
    }
    
    class EventPublisher {
        <<trait>>
        +publish(event: Event)
    }
    
    class ValueObjects {
        <<module>>
        +EntityId
        +Timestamp
        +Tag
        +TimerConfiguration
    }
    
    class Traits {
        <<module>>
        +Readable
        +Writable
        +Searchable
    }
    
    SharedKernel *-- Event
    SharedKernel *-- EventPublisher
    SharedKernel *-- ValueObjects
    SharedKernel *-- Traits
```

### 2. Task Module
Manages task entities and their lifecycle.

```mermaid
classDiagram
    class Task {
        +id: TaskId
        +name: String
        +description: Option~String~
        +max_sessions: u8
        +current_sessions: u8
        +tags: Vec~String~
        +status: Status
        +created_at: DateTime
        +completed_at: Option~DateTime~
        +increment_session()
        +complete()
        +reset_sessions()
    }
    
    class TaskRepository {
        <<trait>>
        +get(id: TaskId) Task
        +save(task: Task)
        +find_all() Vec~Task~
        +delete(id: TaskId)
    }
    
    class CyclingService {
        <<trait>>
        +get_next_task() Option~Task~
        +handle_task_completed(task: Task)
    }
    
    class TaskEvents {
        <<module>>
        +TaskCreated
        +TaskUpdated
        +TaskCompleted
        +TaskSessionCompleted
        +TaskStatusChanged
    }
    
    Task --> TaskRepository
    Task --> CyclingService
    Task --> TaskEvents
```

### 3. Timer Module
Implements the Pomodoro timer state machine.

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> Working: Start
    
    Working --> WorkingPaused: Pause
    WorkingPaused --> Working: Resume
    Working --> ShortBreak: Complete (sessions < 4)
    Working --> LongBreak: Complete (sessions = 4)
    Working --> Idle: Reset
    
    ShortBreak --> ShortBreakPaused: Pause
    ShortBreakPaused --> ShortBreak: Resume
    ShortBreak --> Working: Complete
    ShortBreak --> Idle: Reset
    
    LongBreak --> LongBreakPaused: Pause
    LongBreakPaused --> LongBreak: Resume
    LongBreak --> Idle: Complete
    LongBreak --> Idle: Reset
    
    WorkingPaused --> Idle: Reset
    ShortBreakPaused --> Idle: Reset
    LongBreakPaused --> Idle: Reset
```

### Timer State Machine Implementation

```mermaid
classDiagram
    class TimerState {
        <<enum>>
        +Idle
        +Working
        +WorkingPaused
        +ShortBreak
        +ShortBreakPaused
        +LongBreak
        +LongBreakPaused
    }
    
    class Timer {
        +id: TimerId
        +state: TimerState
        +configuration: TimerConfiguration
        +session_count: u32
        +active_task: Option~TaskId~
        +start()
        +pause()
        +resume()
        +reset()
        +skip_phase()
        +tick()
    }
    
    class StateTransitions {
        +handle_start(state: TimerState) TransitionResult
        +handle_pause(state: TimerState) TransitionResult
        +handle_resume(state: TimerState) TransitionResult
        +handle_reset(state: TimerState) TransitionResult
        +handle_tick(state: TimerState) TransitionResult
        +handle_skip(state: TimerState) TransitionResult
    }
    
    class TimerEvents {
        <<module>>
        +TimerStarted
        +TimerPaused
        +TimerReset
        +TimerTick
        +PhaseCompleted
        +PhaseSkipped
        +WorkPhaseStarted
        +WorkPhaseCompleted
        +BreakPhaseStarted
        +BreakPhaseCompleted
    }
    
    Timer --> TimerState
    Timer --> StateTransitions
    Timer --> TimerEvents
```

## Event Flow

### Timer Lifecycle Events

```mermaid
sequenceDiagram
    participant U as User
    participant T as Timer
    participant E as EventPublisher
    participant Task as Task
    
    U->>T: Start Timer
    T->>E: Publish(TimerStarted)
    T->>E: Publish(WorkPhaseStarted)
    
    loop Every Second
        T->>T: Tick
        T->>E: Publish(TimerTick)
    end
    
    T->>E: Publish(PhaseCompleted)
    T->>Task: Increment Session
    T->>E: Publish(TaskSessionCompleted)
    
    alt Sessions < 4
        T->>E: Publish(BreakPhaseStarted)
    else Sessions = 4
        T->>E: Publish(LongBreakStarted)
    end
```

### Task Completion Flow

```mermaid
sequenceDiagram
    participant T as Timer
    participant Task as Task
    participant CS as CyclingService
    participant E as EventPublisher
    
    T->>Task: Complete Session
    Task->>Task: Increment Sessions
    
    alt Task Complete
        Task->>E: Publish(TaskCompleted)
        Task->>CS: Get Next Task
        CS->>E: Publish(TaskCyclingCompleted)
    else Task Not Complete
        Task->>E: Publish(TaskSessionCompleted)
    end
```

## Domain Boundaries

```mermaid
graph LR
    subgraph "Domain Core"
        T[Timer]
        Task[Task]
        C[Config]
    end
    
    subgraph "Domain Services"
        TS[TimerService]
        CS[CyclingService]
        AS[AudioService]
    end
    
    subgraph "Domain Events"
        TE[Timer Events]
        TaskE[Task Events]
        CE[Command Events]
    end
    
    T --> TS
    Task --> CS
    T --> TE
    Task --> TaskE
    TS --> CE
    CS --> CE
```

## Repository Pattern

```mermaid
classDiagram
    class Repository~T~ {
        <<interface>>
        +get(id: EntityId) Result~T~
        +save(entity: T) Result
        +delete(id: EntityId) Result
    }
    
    class TaskRepository {
        <<trait>>
        +find_by_status(status: Status) Vec~Task~
        +find_incomplete() Vec~Task~
    }
    
    class ConfigRepository {
        <<trait>>
        +get_config() Config
        +update_config(config: Config)
    }
    
    Repository <|-- TaskRepository
    Repository <|-- ConfigRepository
```

## Value Objects

```mermaid
classDiagram
    class TimerConfiguration {
        +work_duration: u32
        +short_break_duration: u32
        +long_break_duration: u32
        +sessions_until_long_break: u32
        +validate() Result
    }
    
    class TaskId {
        <<value object>>
        +value: Uuid
        +new() TaskId
    }
    
    class Timestamp {
        <<value object>>
        +value: DateTime~Utc~
        +now() Timestamp
    }
    
    class Tag {
        <<value object>>
        +value: String
        +validate() Result
    }
```

## Error Handling

```mermaid
graph TB
    E[DomainError]
    E --> TE[TaskError]
    E --> TME[TimerError]
    E --> CE[ConfigError]
    E --> AE[AudioError]
    
    TE --> TC[TaskAlreadyCompleted]
    TE --> TI[InvalidTaskId]
    
    TME --> IS[InvalidState]
    TME --> IT[InvalidTransition]
    
    CE --> IC[InvalidConfiguration]
    CE --> CNF[ConfigNotFound]
```

## Cross-Cutting Concerns

### Event Publishing

All domain modules publish events through the EventPublisher trait, enabling:
- Decoupled communication between modules
- Audit logging
- State synchronization
- UI updates

### Persistence

Domain entities are persistence-agnostic, using repository traits that can be implemented by the infrastructure layer.

### Validation

Each domain entity validates its invariants:
- Tasks must have valid session counts
- Timer states must follow valid transitions
- Configurations must have reasonable durations

## Design Patterns Used

1. **State Pattern**: Timer state machine
2. **Repository Pattern**: Data access abstraction
3. **Factory Pattern**: Task and Timer builders
4. **Strategy Pattern**: Task cycling strategies
5. **Domain Events**: Event-driven architecture
6. **Value Objects**: Immutable domain primitives