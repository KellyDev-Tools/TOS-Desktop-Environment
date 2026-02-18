pub mod input;
pub mod pty;
pub mod ipc;

// Phase 15: Performance Monitoring & Tactical Alerts
pub mod performance;

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
pub mod audio;
pub mod proc;

// Phase 15: Auditory Interface - Earcons and Themes
pub use audio::earcons;
pub use audio::themes;

// Phase 15: Advanced Input (Game Controllers, VR/AR, Hand/Eye Tracking)
pub use input::advanced;

#[cfg(feature = "live-feed")]
pub mod live_feed;

// Phase 16: Container Strategy & SaaS Architecture
// Note: containers module is at crate root (src/containers/)
// Re-exported in lib.rs
