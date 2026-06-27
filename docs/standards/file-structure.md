Domain-Centric Organization

# file creation
```
IF module has > 3 related items:
    CREATE subdirectory
    SPLIT into focused files
ELSE:
    KEEP in single file

IF file has > 150 lines:
    CONSIDER splitting by responsibility
```

# file structure
```
└── domain/                         # DDD Domain Layer
    ├── Cargo.toml                  # itala-domain
    └── src/
        ├── lib.rs
        ├── shared_kernel/          # Shared concepts across domains
        │   ├── mod.rs
        │   ├── events/
        │   │   ├── mod.rs
        │   │   └── event.rs        # pub trait Event
        │   ├── traits/
        │   │   ├── mod.rs
        │   │   ├── readable.rs     # pub trait Readable
        │   │   ├── searchable.rs   # pub trait Searchable
        │   │   └── writable.rs     # pub trait Writable
        │   └── value_objects/
        │       ├── mod.rs
        │       ├── identifier.rs   # pub struct Identifier
        │       ├── location.rs     # pub struct Location
        │       ├── tag.rs          # pub struct Tag
        │       └── timestamp.rs    # pub struct Timestamp
        │
        ├── task/                   # Task Domain (Bounded Context)
        │   ├── mod.rs
        │   ├── entity.rs           # pub struct Task
        │   ├── id.rs               # pub struct Id
        │   ├── status.rs           # pub enum Status
        │   ├── config.rs           # pub struct Config
        │   ├── repository.rs       # pub trait Repository
        │   ├── events.rs           # pub enum Event (or individual event types)
        │   ├── cycle_service.rs    # pub struct CycleService (functionalities)
        │   └── session/            # Session-specific functionality
        │       ├── mod.rs
        │       ├── auth.rs
        │       └── service.rs      # pub struct Service
        │
        ├── timer/                  # Timer Domain (Bounded Context)
        │   ├── mod.rs
        │   ├── phase.rs            # pub enum Phase
        │   ├── state.rs            # pub struct State
        │   ├── status.rs           # pub enum Status
        │   ├── state_with_task.rs  # pub struct StateWithTask
        │   └── phase_transition.rs
        │
        └── config/                 # Config Domain (Bounded Context)
            ├── mod.rs
            └── ...
```

# Usage Examples

```rust
// Import with type aliases for clarity
use crate::domain::task::{
    Task,
    Id as TaskId,
    Status as TaskStatus,
    Repository as TaskRepository,
};

use crate::domain::task::cycling::Service as TaskCyclingService;
use crate::domain::task::session::Service as TaskSessionService;

use crate::domain::timer::{
    Phase,
    State as TimerState,
    StateWithTask as TimerStateWithTask,
};

use crate::domain::timer::phase_transition::Service as PhaseTransitionService;
```