# 🏛️ Architecture Overview

## Clean Architecture Principles

Pomotoro follows **Clean Architecture** principles with a clear separation of concerns:

```mermaid
graph LR
    subgraph "External World"
        USER[User]
        FS[File System]
        AUDIO[Audio System]
        OS[OS Services]
    end
    
    subgraph "Application Boundary"
        subgraph "Presentation"
            UI[UI Components]
            VM[View Models]
        end
        
        subgraph "Application"
            UC[Use Cases]
            CMD[Commands]
        end
        
        subgraph "Domain Core"
            ENT[Entities]
            VO[Value Objects]
            DSRV[Domain Services]
            DEVT[Domain Events]
        end
        
        subgraph "Infrastructure"
            REPO[Repositories]
            ADAPT[Adapters]
            EVTBUS[Event Bus]
        end
    end
    
    USER <--> UI
    UI <--> VM
    VM <--> CMD
    CMD <--> UC
    UC <--> ENT
    UC <--> DSRV
    UC <--> REPO
    REPO <--> FS
    ADAPT <--> AUDIO
    ADAPT <--> OS
    
    style ENT fill:#f9f,stroke:#333,stroke-width:4px
    style UC fill:#bbf,stroke:#333,stroke-width:2px
    style UI fill:#bfb,stroke:#333,stroke-width:2px
```

## The Four Layers

### 1. Domain Layer (`domain/`)
**The Heart of the Application**
- Pure business logic
- No external dependencies
- Contains:
  - Entities (Timer, Task, Config)
  - Value Objects (TaskId, Timestamp)
  - Domain Events
  - Domain Services
  - Repository Interfaces

### 2. Use Cases Layer (`usecases/`)
**Application Business Rules**
- Orchestrates domain entities
- Implements application-specific logic
- Contains:
  - Use case implementations
  - Data mappers
  - Service interfaces

### 3. Infrastructure Layer (`infra/`)
**External World Integration**
- Implements repository interfaces
- Handles external services
- Contains:
  - File repositories
  - Memory repositories
  - Event bus implementation
  - Audio service adapters
  - Tauri command handlers

### 4. UI Layer (`apps/react-ui/`)
**User Interface**
- React + TypeScript components
- View models
- State management
- User interaction handling

## Dependency Rules

```mermaid
graph TD
    UI[UI Layer]
    UC[Use Cases Layer]
    D[Domain Layer]
    I[Infrastructure Layer]
    
    UI --> UC
    UI --> I
    UC --> D
    I --> D
    I --> UC
    
    UI -.->X[Cannot depend on] -.-> D
    D -.->X -.-> UC
    D -.->X -.-> I
    D -.->X -.-> UI
    
    style D fill:#f9f,stroke:#333,stroke-width:4px
    style X fill:#fcc,stroke:#f00,stroke-width:2px,stroke-dasharray: 5 5
```

### The Dependency Rule
Dependencies only point **inward**:
- Domain knows nothing about outer layers
- Use Cases know Domain but not Infrastructure/UI
- Infrastructure and UI can know about inner layers

## Key Architectural Patterns

### 1. Repository Pattern
```rust
// Domain defines interface
trait TaskRepository {
    fn save(&self, task: Task) -> Result<()>;
    fn find(&self, id: TaskId) -> Result<Option<Task>>;
}

// Infrastructure implements
struct FileTaskRepository { ... }
impl TaskRepository for FileTaskRepository { ... }
```

### 2. Event-Driven Architecture
```rust
// Domain event
struct TaskCompleted {
    task_id: TaskId,
    completed_at: Timestamp,
}

// Event handling
EventBus::publish(TaskCompleted { ... });
```

### 3. Command Pattern
```rust
// Tauri command
#[tauri::command]
async fn start_timer(state: State<'_, AppState>) -> Result<()> {
    let use_case = StartTimerSession::new(state.timer_service());
    use_case.execute().await
}
```

## Module Organization

```
pomotoro/
├── domain/           # Core business logic
│   ├── timer/       # Timer aggregate
│   ├── task/        # Task aggregate
│   ├── config/      # Configuration aggregate
│   └── shared_kernel/ # Shared types
├── usecases/        # Application logic
│   ├── timer/       # Timer use cases
│   ├── task/        # Task use cases
│   └── config/      # Config use cases
├── infra/           # External adapters
│   ├── adapters/    # Repository implementations
│   ├── commands/    # Tauri commands
│   └── bootstrap.rs # Dependency injection
└── ui/              # Presentation
    ├── components/  # Reusable UI components
    ├── pages/       # Page components
    └── store/       # State management
```

## Data Flow Example

Let's trace a "Start Timer" action:

```mermaid
sequenceDiagram
    participant User
    participant UI
    participant Command
    participant UseCase
    participant Domain
    participant Repository
    participant EventBus
    
    User->>UI: Click Start Button
    UI->>Command: invoke("start_timer")
    Command->>UseCase: StartTimerSession::execute()
    UseCase->>Domain: Timer::start()
    Domain->>Domain: Validate & Update State
    Domain-->>EventBus: Publish(TimerStarted)
    UseCase->>Repository: save(timer)
    Repository-->>UseCase: Ok(())
    UseCase-->>Command: Ok(TimerState)
    Command-->>UI: Update View
    UI-->>User: Show Running Timer
```

## Testing Strategy

Each layer has its own testing approach:

1. **Domain**: Unit tests with in-memory implementations
2. **Use Cases**: Integration tests with test doubles
3. **Infrastructure**: Integration tests with real services
4. **UI**: Component tests and E2E tests

## Next Steps

Now that you understand the architecture:
1. Read [Getting Started](./getting-started.md) to set up your environment
2. Explore specific [Layer Guides](../layers/)
3. Learn about [Data Flow](../connections/data-flow.md)