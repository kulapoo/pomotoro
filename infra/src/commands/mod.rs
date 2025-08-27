//! Commands Layer
//!
//! This layer contains the command handlers that serve as the entry point
//! for external requests. Commands are responsible for:
//!
//! - Handling incoming requests (Tauri commands, HTTP, etc.)
//! - Input validation and sanitization
//! - Delegating business logic to Application Services
//! - Converting responses to appropriate formats
//! - Error handling and response formatting
//!
//! ## Clean Architecture Principles
//!
//! Commands are in the **outermost layer** and depend on the Application Layer.
//! They should NOT contain business logic - only request/response handling.
//!
//! **Dependency Flow**: Commands → Application → Domain → Infrastructure
//!
//! ## Structure
//!
//! - `task_cmd`: Task management request handlers
//! - `timer_cmd`: Timer control request handlers
//! - `config_cmd`: Configuration management request handlers
//! - `audio_cmd`: Audio system request handlers

pub mod audio_cmd;
pub mod config_cmd;
pub mod notification_cmd;
pub mod storage_cmd;
pub mod task_cmd;
pub mod task_settings_cmd;
pub mod timer_cmd;

pub use audio_cmd::*;
pub use config_cmd::*;
pub use notification_cmd::*;
pub use storage_cmd::*;
pub use task_cmd::*;
pub use task_settings_cmd::*;
pub use timer_cmd::*;
