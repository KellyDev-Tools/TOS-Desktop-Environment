//! TOS Common — universal foundation for TOS.
//!
//! This crate contains shared types, platform traits, and core orchestration logic
//! used by the Brain and all Face implementations.

pub mod collaboration;
pub mod ipc;
pub mod state;
pub mod marketplace;
pub mod modules;
pub mod platform;
pub mod shell;

// Migrate core Brain/Face logic into common foundation
pub mod brain;
pub mod config;
pub mod daemon;
pub mod face;
pub mod services;

#[cfg(test)]
mod brain_tests;

// Re-export core types
pub use collaboration::*;
pub use ipc::IpcDispatcher;
pub use marketplace::*;
pub use modules::*;
pub use platform::*;
pub use shell::*;
pub use state::*;
