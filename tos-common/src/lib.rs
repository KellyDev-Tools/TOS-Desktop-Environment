//! TOS Common — universal foundation for TOS.

pub mod collaboration;
pub mod ipc;
pub mod keybindings;
pub mod state;
pub mod marketplace;
pub mod modules;
pub mod platform;
pub mod shell;

// Brain orchestrator logic
pub mod brain;
pub mod config;
pub mod daemon;
pub mod face;
pub mod services;

pub use face::{Face, MockFace};
// mod brain_tests;

pub use config::TosConfig;
pub use state::{
    TosState, Sector, CommandHub, CommandHubMode,
    TerminalOutputModuleMeta, TerminalLayoutType, ThemeModule, ThemeAssetDefinition,
    AiModuleMetadata, TerminalContext, TerminalLine, SettingsStore, HierarchyLevel,
    TrustTier, ConfirmationRequest, ApplicationModel, BezelAction, DecorationPolicy,
    ZoomBehavior, AppInstance, SectorTemplate, HubTemplate, DirectoryListing,
    DirectoryEntry, ActivityListing, ProcessEntry, SearchResult, AiMessage,
    SplitOrientation, PaneContent, SplitPane, SplitNode, AiBehavior,
    EditorPaneState, EditorMode, DiffHunk, EditorAnnotation,
    AiThought, AiThoughtStatus,
    KanbanBoard, KanbanLane, KanbanTask, KanbanTaskStatus
};

pub use modules::{AiModule, AiQuery, AiResponse, ShellModule, ShellIntegration};
pub use ipc::IpcDispatcher;
pub use marketplace::{
    MarketplaceHome, MarketplaceCategory, MarketplaceModuleSummary, 
    MarketplaceModuleDetail, InstallProgress, MarketplaceReview
};
pub use services::marketplace::{ModuleManifest, ExecutableConfig, MarketplaceService};
pub use collaboration::{Participant, ParticipantRole, PresenceStatus, WebRtcPayload};
pub use platform::{AppPlatform, PlatformStatus, RemoteServer, remote::RemoteRenderer};
pub use shell::{OscEvent, OscParser};
pub use keybindings::{KeyCombo, Keybinding, KeybindingMap};
#[cfg(feature = "test-utils")]
pub use daemon::MockBrain;
pub use daemon::{register_with_brain, log_to_brain, install_crash_handler};
