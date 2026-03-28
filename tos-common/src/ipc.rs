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

/// The platform profile of a connecting Face.
///
/// Profiles define the default layout, input handlers, and AI skill
/// priorities for a given device class (§4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaceProfile {
    /// Large-screen, mouse/keyboard first (Laptop/Desktop).
    Desktop,
    /// touch-first, portrait/landscape (Phone/Tablet).
    Handheld,
    /// XR/AR/VR, raycast/spatial input (Quest/VisionPro).
    Spatial,
    /// Minimal headless interface (CLI/Daemon).
    Headless,
}

/// Initial registration message sent by every Face upon connection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FaceRegister {
    pub face_id: uuid::Uuid,
    pub profile: FaceProfile,
    pub version: String,
}

/// Dynamic registration request for satellite daemons.
///
/// Every service (§4.1) MUST register with the Brain on `brain.sock`
/// before it can be discovered by other components.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceRegister {
    /// Daemon name (e.g. "tos-settingsd").
    pub name: String,
    /// The ephemeral port the daemon is listening on.
    pub port: u16,
    /// Hex-encoded ed25519 signature of the registration payload.
    pub signature: String,
    /// Hex-encoded public key used for verification.
    pub public_key: String,
}

/// Response sent by the Brain after processing a service registration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceRegisterResponse {
    pub status: String,
    pub message: String,
}

/// §14.1: Semantic Input Abstraction
///
/// Every physical input event (key down, touch, voice) is normalized into
/// a SemanticEvent before being dispatched to the Brain logic core.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SemanticEvent {
    // Navigation
    ZoomIn,
    ZoomOut,
    Home,
    CommandHub,
    SwitchSector(usize),

    // Selection
    Select(String),
    SecondarySelect(String),

    // Mode Control
    SetMode(crate::state::CommandHubMode),
    ToggleHiddenFiles,

    // Bezel & View
    ToggleBezel,
    SplitView {
        orientation: String,
        override_auto: bool,
    },
    CloseViewport(uuid::Uuid),
    Inspect(String),
    ToggleMinimap,

    // Text & AI
    PromptSubmit(String),
    PromptStage(String),
    AiSubmit(String),
    AiStop,
    AiSuggestionAccept,

    // System
    SectorCreate(String),
    TacticalResetSector(uuid::Uuid),
    TacticalResetSystem,
    StopOperation,
}
