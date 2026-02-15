pub mod input;
pub mod pty;
pub mod ipc;

// Phase 11: Enhanced Tactical Reset
pub mod reset;

// Phase 11: Voice Command System
pub mod voice;

// Phase 11: Shell API
pub mod shell_api;
pub mod shell;

// Phase 11: Security and Dangerous Command Handling
pub mod security;

// Phase 12: Remote Sectors & Collaboration
pub mod remote;
pub mod collaboration;

#[cfg(feature = "live-feed")]
pub mod live_feed;
