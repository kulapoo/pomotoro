//! Controllers Layer
//!
//! This layer contains the controllers (request handlers) that serve as the entry point
//! for external requests. Controllers are responsible for:
//!
//! - Handling incoming requests (Tauri commands, HTTP, etc.)
//! - Input validation and sanitization
//! - Delegating business logic to Application Services
//! - Converting responses to appropriate formats
//! - Error handling and response formatting
//!
//! ## Clean Architecture Principles
//!
//! Controllers are in the **outermost layer** and depend on the Application Layer.
//! They should NOT contain business logic - only request/response handling.
//!
//! **Dependency Flow**: Controllers → Application → Domain → Infrastructure
//!
//! ## Structure
//!
//! - `task_ctrl`: Task management request handlers
//! - `timer_ctrl`: Timer control request handlers
//! - `config_ctrl`: Configuration management request handlers
//! - `audio_ctrl`: Audio system request handlers

pub mod task_ctrl;
pub mod timer_ctrl;
pub mod config_ctrl;
pub mod audio_ctrl;

pub use task_ctrl::*;
pub use timer_ctrl::*;
pub use config_ctrl::*;
pub use audio_ctrl::*;