pub mod input;
pub mod pty;
pub mod ipc;

// Performance Monitoring & Tactical Alerts features
pub mod performance;

// Enhanced Tactical Reset features
pub mod reset;

// Voice Command System implementation
pub mod voice;

// Shell API implementation
pub mod shell_api;
pub mod shell;
pub mod ai;
pub mod search;

// Security and Dangerous Command Handling implementation
pub mod security;

// Remote Sectors & Collaboration implementation
pub mod remote;
pub mod collaboration;
pub mod audio;
pub mod proc;

// Auditory Interface - Earcons and Themes implementation
pub use audio::earcons;
pub use audio::themes;

// Advanced Input implementation
pub use input::advanced;

#[cfg(feature = "live-feed")]
pub mod live_feed;

// Container Strategy & SaaS Architecture implementation
// Note: containers module is at crate root (src/containers/)
// Re-exported in lib.rs
