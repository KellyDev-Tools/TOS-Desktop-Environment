// Re-export core platform traits and types from tos-common
pub use tos_common::platform::*;

pub mod headless;
pub mod mock;
pub mod quest;
pub mod remote;
pub mod remote_server;
pub mod remote_session;
pub mod ssh_fallback;

pub use headless::HeadlessRenderer;
pub use remote::RemoteRenderer;
pub use remote_server::RemoteServer;
