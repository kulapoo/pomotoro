//! Infrastructure Layer
//!
//! This layer contains concrete implementations of domain interfaces and handles
//! external concerns like persistence, event publishing, notifications, and 
//! integration with external systems.
//!
//! ## Clean Architecture Principles
//!
//! - **Implementation**: Concrete implementations of domain abstractions
//! - **External Concerns**: Database, file system, network, UI frameworks
//! - **Dependency Direction**: Depends on domain abstractions, not implementations
//! - **Isolation**: Changes here shouldn't affect domain logic
//!
//! ## Domain-Based Structure
//!
//! - `config/`: Configuration domain infrastructure
//! - `task/`: Task domain infrastructure  
//! - `timer/`: Timer domain infrastructure
//! - `audio/`: Audio domain infrastructure
//! - `events/`: Event publishing infrastructure
//! - `notifications`: System notification integration

pub mod config;
pub mod task;
pub mod timer;
pub mod audio;
pub mod events;
pub mod notifications;

pub use config::*;
pub use task::*;
pub use timer::*;
pub use audio::*;
pub use events::*;
pub use notifications::*;