//! TOS Protocol — shared types for the Face-Brain communication contract.
//!
//! This crate owns every type that crosses the boundary between the Brain
//! (logic core) and any Face (display layer). By extracting these into a
//! standalone crate, both sides can depend on a stable, versioned contract
//! without importing each other's internals.
//!
//! # Modules
//!
//! - [`state`] — The system-wide `TosState` and all nested structs.
//! - [`ipc`] — IPC dispatcher trait and message contracts.
//! - [`modules`] — Module trait contracts (AI, Shell, Terminal Output).
//! - [`collaboration`] — Multi-user collaboration payloads (WebRTC).

pub mod ipc;
pub mod state;
pub use ipc as ipc_dispatcher;
pub mod collaboration;
pub mod marketplace;
pub mod modules;
pub mod platform;
pub mod shell;

// Re-export core types at the crate root for ergonomic access.
pub use collaboration::{Participant, ParticipantRole, PresenceStatus, WebRtcPayload};
pub use ipc::IpcDispatcher;
pub use marketplace::*;
pub use modules::{AiModule, AiQuery, AiResponse, ShellIntegration, ShellModule};
pub use platform::*;
pub use shell::*;
pub use state::*;
