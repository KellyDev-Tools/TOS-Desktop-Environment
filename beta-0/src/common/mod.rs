//! Common types for the TOS Brain — re-exported from the `tos-protocol` crate.
//!
//! This module exists as a compatibility bridge. All shared types are
//! defined in `tos-protocol` and re-exported here so that existing Brain
//! code continues to compile without changes to import paths.

// Re-export the entire state module — this gives access to TosState,
// Sector, CommandHub, SettingsStore, HierarchyLevel, CommandHubMode,
// and every other shared type.
pub use tos_protocol::state::*;

/// Backward-compatible alias — the protocol crate renamed the struct to
/// `TerminalOutputModuleMeta` to avoid collision with the trait of the
/// same name. Existing Brain code can continue using this alias.
pub type TerminalOutputModule = tos_protocol::state::TerminalOutputModuleMeta;

// Re-export the IPC dispatcher trait.
pub use tos_protocol::ipc;
pub mod ipc_dispatcher {
    pub use tos_protocol::ipc::IpcDispatcher;
}

// Re-export module contracts under the `modules` namespace.
pub mod modules {
    pub use tos_protocol::modules::*;
}

// Re-export collaboration types under the `collaboration` namespace.
pub mod collaboration {
    pub use tos_protocol::collaboration::*;
}
