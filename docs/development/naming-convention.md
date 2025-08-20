# Context-Aware Naming Convention

## Core Principle
All types use generic names - module paths provide context. Use type aliases when importing for clarity.

## File Naming
```rust
✅ CORRECT
domain/task/error.rs
domain/task/events/completed.rs
domain/task/repository.rs
domain/user/service.rs

❌ WRONG
domain/task/task_error.rs           // Redundant prefix
domain/task/events/task_completed.rs // Redundant prefix
```

## Type Naming - Always Generic
```rust
// In domain/task/error.rs
pub enum Error {
    NotFound,
    InvalidState,
}

// In domain/task/repository.rs
pub trait Repository {
    async fn find(&self, id: Id) -> Result<Task, Error>;
}

// In domain/task/events/completed.rs
pub struct Completed {
    id: Id,
    at: DateTime,
}

// In domain/task/commands/create.rs
pub struct Create {
    title: String,
    description: String,
}
```

## Import Pattern - Use Type Aliases

### Within Module Context
```rust
use crate::domain::task;

let err = task::Error::NotFound;
let event = task::events::Completed { ... };
```

### Cross-Module with Type Aliases
```rust
use crate::domain::task::Error as TaskError;
use crate::domain::task::Repository as TaskRepository;
use crate::domain::task::events::Completed as TaskCompleted;
use crate::domain::task::commands::Create as CreateTask;

use crate::domain::user::Error as UserError;
use crate::domain::user::Repository as UserRepository;

match result {
    Err(TaskError::NotFound) => {},
    Err(UserError::Unauthorized) => {},
}
```

## Why This Pattern?

**Consistency** - One rule for everything
**No Stuttering** - Never see `task::TaskError`
**Follows Rust stdlib** - `std::io::Error`, not `IoError`
**Explicit at Use Site** - Type aliases make intent clear where it matters

## Handling Specialized Components

**Pragmatic Approach**: Start simple with descriptive files. Only create submodules when you have multiple related types that benefit from grouping.

### Option 1: Descriptive Files with Descriptive Types (Start Here)
```rust
domain/
  task/
    cycle_service.rs         // pub struct CycleService
    priority_calculator.rs   // pub struct PriorityCalculator

// Usage
use crate::domain::task::cycle_service::CycleService;
use crate::domain::task::priority_calculator::PriorityCalculator;
```

**When to use**: Default choice for specialized components. Simple, flat, and avoids premature abstraction.

### Option 2: Submodules with Generic Names (When Complexity Justifies)
```rust
domain/
  task/
    cycle/
      service.rs     // pub struct Service
      validator.rs   // pub struct Validator
      state.rs       // pub enum State
    priority/
      calculator.rs  // pub struct Calculator
      queue.rs       // pub struct Queue
      strategy.rs    // pub trait Strategy

// Usage
use crate::domain::task::cycle::Service as CycleService;
use crate::domain::task::priority::Calculator as PriorityCalculator;
```

**When to use**: When you have multiple related types (3+) that form a cohesive subsystem.

**Rule**: Start with descriptive files (Option 1). Refactor to submodules (Option 2) only when you have multiple related types that benefit from grouping. Avoid creating a submodule for a single type.

## Example Module Structure
```rust
domain/
  task/
    error.rs               // pub enum Error
    repository.rs          // pub trait Repository
    service.rs             // pub struct Service (main task service)
    cycle_service.rs       // pub struct CycleService (single specialized component)
    priority_calculator.rs // pub struct PriorityCalculator (single specialized component)
    events/
      completed.rs         // pub struct Completed
      started.rs           // pub struct Started
    commands/
      create.rs            // pub struct Create
      update.rs            // pub struct Update
```

## Public API Exports
```rust
// In domain/task/mod.rs
pub use self::error::Error;
pub use self::repository::Repository;
pub use self::service::Service;
pub use self::cycle_service::CycleService;
pub use self::priority_calculator::PriorityCalculator;

pub mod events {
    pub use super::events::completed::Completed;
    pub use super::events::started::Started;
}

pub mod commands {
    pub use super::commands::create::Create;
    pub use super::commands::update::Update;
}
```

## Library Consumer Usage
```rust
// External crate using your library
use your_crate::domain::task::{
    Error as TaskError,
    Repository as TaskRepository,
    CycleService,
    PriorityCalculator,
    events::Completed as TaskCompleted,
    commands::Create as CreateTask,
};
```