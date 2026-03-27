//! IPC dispatcher trait — the contract for handling Brain-bound messages.
//!
//! Any component that processes IPC messages (the Brain's `IpcHandler`,
//! test harnesses, mock dispatchers) implements this trait.

/// Synchronous dispatcher for Brain-bound IPC messages.
///
/// The Face serializes user actions into string messages and sends them
/// over TCP or WebSocket. The Brain deserializes, dispatches via this
/// trait, and returns a string response.
pub trait IpcDispatcher: Send + Sync {
    /// Route a raw IPC request string and return the response.
    fn dispatch(&self, request: &str) -> String;
}
