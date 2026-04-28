//! System-wide state types for the Brain core logic process.
//!
//! Every struct in this module is serialized over the WebSocket state sync
//! channel and must remain stable across Face and Brain versions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// The system hierarchy levels defining the visual depth of the interface.
///
/// Navigation is strictly vertical — zoom in or zoom out. There is no
/// lateral navigation at the same level (§2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchyLevel {
    GlobalOverview = 1,
    CommandHub = 2,
    ApplicationFocus = 3,
    DetailView = 4,
    BufferView = 5,
    Marketplace = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanningMode {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityState {
    pub scanning_enabled: bool,
    pub scanning_mode: ScanningMode,
    pub scanning_interval_ms: u32,
    pub active_scan_path: Vec<String>,
    pub current_scan_index: usize,
}

impl Default for AccessibilityState {
    fn default() -> Self {
        Self {
            scanning_enabled: false,
            scanning_mode: ScanningMode::Auto,
            scanning_interval_ms: 1000,
            active_scan_path: vec![],
            current_scan_index: 0,
        }
    }
}

/// Context for terminal module execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalContext {
    Interactive,
    ReadOnly,
}

/// Metadata for installable terminal rendering modules.
///
/// Each module defines a visual language for rendering terminal output.
/// The active module is selected in Settings and persisted per-sector.
pub type TerminalOutputModule = TerminalOutputModuleMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputModuleMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub layout: TerminalLayoutType,
    pub supports_high_contrast: bool,
    pub supports_reduced_motion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellModuleMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub executable: String,
    pub integration: crate::modules::ShellIntegration,
    pub scripts: Vec<String>,
    pub init: String,
}

/// Layout geometry for terminal output rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalLayoutType {
    Rectangular,
    Cinematic,
}

/// Metadata for installable theme modules.
///
/// Themes are static asset bundles that define the visual appearance of
/// the entire interface — colours, fonts, and icon sets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeModule {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub assets: ThemeAssetDefinition,
}

/// Asset paths for a theme module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeAssetDefinition {
    pub css: String,
    pub fonts: Vec<String>,
    pub icons: String,
}

/// Metadata for installable AI backend modules.
///
/// AI backends define the LLM connection — model, endpoint, auth.
/// Multiple backends can be installed simultaneously; one is the system default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModuleMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

/// A registered AI behavior module instance in the behavior registry.
///
/// Each behavior declares the context fields it needs and optionally
/// overrides the system-default AI backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBehavior {
    /// Behavior type ID (e.g. "command_autocomplete", "ghost_suggestions").
    pub id: String,
    /// Human-readable label.
    pub name: String,
    /// Whether this behavior is currently active.
    pub enabled: bool,
    /// Optional per-behavior backend override (module ID). None → uses system default.
    pub backend_override: Option<String>,
    /// Context fields this behavior has declared it needs.
    pub context_fields: Vec<String>,
    /// Authorized tools in the tool bundle.
    pub allowed_tools: Option<Vec<String>>,
    /// Arbitrary configuration key-value pairs.
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BezelComponentState {
    pub id: String,
    pub name: String,
    pub html: String,
    pub data: serde_json::Value,
    pub slot: String, // "top", "left", "right"
}

/// The operational augmentation modes for the Command Hub.
///
/// Each mode changes the chip column content and terminal output rendering
/// while keeping the Persistent Unified Prompt always available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandHubMode {
    Command,
    Directory,
    Activity,
    Search,
    Ai,
    Ssh,
}

/// Defines the security trust level for operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustTier {
    /// Sandboxed execution — limited permissions.
    Standard,
    /// Privileged execution — full system access.
    System,
}

/// Security validation for execution of dangerous system commands.
///
/// Retained for backward compatibility during the transition to the new
/// Trust & Confirmation System. The `progress` field is deprecated and
/// will be removed once the confirmation slider is fully retired.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    pub id: Uuid,
    pub original_request: String,
    pub message: String,
    pub progress: f32,
}

/// Blueprint for custom application integration at Level 3.
///
/// Applications declare their bezel actions, decoration policy, and zoom
/// behaviour so the Face can render them consistently within the hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationModel {
    pub id: String,
    pub name: String,
    pub version: String,
    pub icon: String,
    pub bezel_actions: Vec<BezelAction>,
    pub decoration_policy: DecorationPolicy,
    pub zoom_behavior: ZoomBehavior,
    pub searchable_content: bool,
}

/// A single action button rendered in the bezel alongside an active application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BezelAction {
    pub label: String,
    pub icon: String,
    pub command: String,
}

/// Controls how window decorations are handled for Level 3 applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecorationPolicy {
    Suppress,
    Overlay,
    Native,
}

/// Controls how zoom gestures interact with Level 3 applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoomBehavior {
    Internal,
    System,
}

/// An active instance of an application within a sector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInstance {
    pub id: Uuid,
    pub model_id: String,
    pub title: String,
    pub state_summary: String,
}

/// Blueprint for creating pre-configured workspaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorTemplate {
    pub name: String,
    pub description: String,
    pub environment: HashMap<String, String>,
    pub hubs: Vec<HubTemplate>,
}

/// Template for a single Command Hub within a sector template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubTemplate {
    pub mode: CommandHubMode,
    pub cwd: String,
    pub shell: String,
}

/// A sector — the primary workspace unit in the TOS hierarchy.
///
/// Each sector is an independent workspace containing one or more Command
/// Hubs, running applications, and collaboration state. Sectors are
/// visible as tiles at Level 1 (Global Overview).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: Uuid,
    pub name: String,
    pub hubs: Vec<CommandHub>,
    pub active_hub_index: usize,
    /// Freeze stops UI updates for this sector.
    pub frozen: bool,
    /// Whether this sector is connected to a remote Brain.
    pub is_remote: bool,
    /// Connection health for remote sectors.
    pub disconnected: bool,
    pub trust_tier: TrustTier,
    /// Tactical priority rank (1–5).
    pub priority: u8,
    pub active_apps: Vec<AppInstance>,
    pub active_app_index: usize,
    /// Multi-user collaboration participants (§13).
    pub participants: Vec<crate::collaboration::Participant>,
    pub kanban_board: Option<KanbanBoard>,
    pub version: u64,
}

/// The Command Hub — the core interactive terminal surface within a sector.
///
/// Combines the Persistent Unified Prompt, chip columns, and the terminal
/// output area into a single composite widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHub {
    pub id: Uuid,
    pub mode: CommandHubMode,
    pub prompt: String,
    pub current_directory: PathBuf,
    pub terminal_output: Vec<TerminalLine>,
    pub buffer_limit: usize,
    /// Directory listing populated in DIR mode.
    pub shell_listing: Option<DirectoryListing>,
    /// Process listing populated in ACT mode.
    pub activity_listing: Option<ActivityListing>,
    /// Search results populated in SEARCH mode.
    pub search_results: Option<Vec<SearchResult>>,
    /// AI-proposed command staged for user review.
    pub staged_command: Option<String>,
    /// AI rationale or documentation for the staged command.
    pub ai_explanation: Option<String>,
    /// Custom JSON exported by the shell via OSC 9004.
    pub json_context: Option<serde_json::Value>,
    /// Preferred shell module identifier.
    pub shell_module: Option<String>,
    pub split_layout: Option<SplitNode>,
    pub focused_pane_id: Option<Uuid>,
    pub version: u64,
    pub ai_history: Vec<AiMessage>,
    pub active_thoughts: Vec<AiThought>,
    pub last_exit_status: Option<i32>,
    pub is_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiThoughtStatus {
    Thinking,
    Decided,
    Actioned,
    Failed,
}

/// A "Thought Bubble" or "Decision Chip" representing an AI's internal reasoning or plan step (§3.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiThought {
    pub id: Uuid,
    pub behavior_id: String,
    pub title: String,
    pub content: String,
    pub status: AiThoughtStatus,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// A "Thought Bubble" or "Decision Chip" representing an AI's internal reasoning or plan step (§3.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub agent_id: String,
    pub task_id: Uuid,
    pub status: AgentStatus,
    pub current_step: usize,
    pub total_steps: usize,
    pub terminal_pane_id: Option<Uuid>,
    pub sandbox_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
}

/// An AI request queued while the system is offline or backend is unreachable (§4.9).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedAiRequest {
    pub behavior_id: String,
    pub prompt: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

// ---------------------------------------------------------------------------
// Kanban & Project Orchestration (§7.2)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanBoard {
    pub project_id: Uuid,
    pub title: String,
    pub lanes: Vec<KanbanLane>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanLane {
    pub id: Uuid,
    pub title: String,
    pub tasks: Vec<KanbanTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanTask {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: KanbanTaskStatus,
    pub assignee: Option<String>,
    pub priority: u8,
    pub tags: Vec<String>,
    /// Staged changes generated by an agent in a sandbox.
    pub staged_changes: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KanbanTaskStatus {
    Todo,
    InProgress,
    Review,
    Done,
    Blocked,
}

// ---------------------------------------------------------------------------
// Split Pane Tree
// ---------------------------------------------------------------------------

/// Orientation of a split container node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitOrientation {
    /// Left/right split (children are stacked side-by-side).
    Vertical,
    /// Top/bottom split (children are stacked above/below).
    Horizontal,
}

/// The mode an editor pane is operating in (§6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
    /// Read-only file view (default on open).
    Viewer,
    /// Full editing with syntax highlighting and input.
    Editor,
    /// Side-by-side diff of pending AI proposal or VCS changes.
    Diff,
}

/// A single diff hunk for Diff Mode rendering (§6.6.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffHunk {
    /// 1-indexed starting line in the original file.
    pub old_start: usize,
    /// Number of lines removed.
    pub old_count: usize,
    /// 1-indexed starting line in the modified file.
    pub new_start: usize,
    /// Number of lines added.
    pub new_count: usize,
    /// The unified-diff text for this hunk.
    pub content: String,
}

/// An inline annotation bound to a specific line in an editor pane (§6.5.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorAnnotation {
    /// 0-indexed line number where the annotation applies.
    pub line: usize,
    /// Severity level: "info", "warning", "error", or "ai".
    pub severity: String,
    /// The message to display.
    pub message: String,
}

/// Persistent state for an editor pane surface (Features §6).
///
/// Serialized into the split pane tree and included in session snapshots.
/// The Face renders a code editing surface from this data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorPaneState {
    /// Absolute path to the file being edited.
    pub file_path: PathBuf,
    /// The full file content (or the visible window for large files).
    pub content: String,
    /// Current editing mode.
    pub mode: EditorMode,
    /// Language identifier for syntax highlighting (e.g. "rust", "python").
    pub language: Option<String>,
    /// 0-indexed cursor line position.
    pub cursor_line: usize,
    /// 0-indexed cursor column position.
    pub cursor_col: usize,
    /// First visible line for scroll-position persistence.
    pub scroll_offset: usize,
    /// Whether the buffer has unsaved modifications.
    pub dirty: bool,
    /// Diff hunks when in Diff mode (AI proposal or VCS).
    pub diff_hunks: Vec<DiffHunk>,
    /// Inline margin annotations (AI remarks, compilation errors) associated with lines.
    pub annotations: Vec<EditorAnnotation>,
}

impl EditorPaneState {
    /// Create a new editor pane in Viewer mode for the given file.
    pub fn new_viewer(path: PathBuf, content: String, language: Option<String>) -> Self {
        Self {
            file_path: path,
            content,
            mode: EditorMode::Viewer,
            language,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            dirty: false,
            diff_hunks: vec![],
            annotations: vec![],
        }
    }
}

/// The type of content a leaf pane contains.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaneContent {
    Terminal,
    Application(String),
    /// A code editor surface (§6, §11.2).
    Editor(EditorPaneState),
    /// A project workflow/Kanban board surface defined in §7.
    Workflow,
}

/// A leaf pane in the split tree — an independently rendered terminal surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPane {
    pub id: Uuid,
    /// Portion of the parent container (0.0–1.0). Siblings must sum to 1.0.
    pub weight: f32,
    pub cwd: PathBuf,
    pub content: PaneContent,
}

/// A recursive split tree node — either a container (with children) or a leaf pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitNode {
    Leaf(SplitPane),
    Container {
        orientation: SplitOrientation,
        children: Vec<SplitNode>,
    },
}

impl SplitNode {
    pub fn all_pane_ids(&self) -> Vec<Uuid> {
        match self {
            SplitNode::Leaf(p) => vec![p.id],
            SplitNode::Container { children, .. } => {
                children.iter().flat_map(|c| c.all_pane_ids()).collect()
            }
        }
    }

    pub fn pane_count(&self) -> usize {
        self.all_pane_ids().len()
    }

    /// Determine ideal orientation from display aspect ratio.
    pub fn ideal_orientation(display_width: u32, display_height: u32) -> SplitOrientation {
        if display_width >= display_height {
            SplitOrientation::Vertical
        } else {
            SplitOrientation::Horizontal
        }
    }

    /// Returns true if a new split is geometrically safe given display dimensions.
    pub fn can_split(pane_count: usize, display_width: u32, display_height: u32) -> bool {
        let new_count = (pane_count + 1).max(1) as u32;
        let min_fraction = 1.0_f32 / 6.0;
        let min_w = ((display_width as f32 * min_fraction) as u32).max(400);
        let min_h = ((display_height as f32 * min_fraction) as u32).max(200);
        (display_width / new_count) >= min_w && (display_height / new_count) >= min_h
    }

    /// Find a SplitPane by UUID (mutable). Traverses the tree recursively.
    pub fn find_pane_mut(&mut self, id: Uuid) -> Option<&mut SplitPane> {
        match self {
            SplitNode::Leaf(ref mut p) if p.id == id => Some(p),
            SplitNode::Leaf(_) => None,
            SplitNode::Container { children, .. } => {
                children.iter_mut().find_map(|c| c.find_pane_mut(id))
            }
        }
    }

    /// Find the first EditorPaneState whose file_path matches the given path string.
    pub fn find_editor_by_path_mut(&mut self, path: &str) -> Option<&mut EditorPaneState> {
        match self {
            SplitNode::Leaf(ref mut pane) => {
                if let PaneContent::Editor(ref mut ed) = pane.content {
                    if ed.file_path.to_string_lossy() == path {
                        return Some(ed);
                    }
                }
                None
            }
            SplitNode::Container { children, .. } => {
                children.iter_mut().find_map(|c| c.find_editor_by_path_mut(path))
            }
        }
    }

    /// Retrieve all active editor panes within this layout.
    pub fn all_editors(&self) -> Vec<&EditorPaneState> {
        match self {
            SplitNode::Leaf(pane) => {
                if let PaneContent::Editor(ed) = &pane.content {
                    vec![ed]
                } else {
                    vec![]
                }
            }
            SplitNode::Container { children, .. } => {
                let mut res = vec![];
                for c in children {
                    res.extend(c.all_editors());
                }
                res
            }
        }
    }

    /// Add a new pane to the split tree. If the tree is a single leaf,
    /// wraps it in a Container with the new pane. If it's already a Container,
    /// appends the new pane and equalizes weights.
    pub fn add_pane(&mut self, pane: SplitPane) {
        match self {
            SplitNode::Leaf(_) => {
                let existing = std::mem::replace(
                    self,
                    SplitNode::Container {
                        orientation: SplitOrientation::Vertical,
                        children: vec![],
                    },
                );
                if let SplitNode::Container { ref mut children, .. } = self {
                    children.push(existing);
                    children.push(SplitNode::Leaf(pane));
                    // Equalize weights
                    let w = 1.0 / children.len() as f32;
                    for child in children.iter_mut() {
                        if let SplitNode::Leaf(ref mut p) = child {
                            p.weight = w;
                        }
                    }
                }
            }
            SplitNode::Container { children, .. } => {
                children.push(SplitNode::Leaf(pane));
                let w = 1.0 / children.len() as f32;
                for child in children.iter_mut() {
                    if let SplitNode::Leaf(ref mut p) = child {
                        p.weight = w;
                    }
                }
            }
        }
    }

    pub fn promote_pane(&mut self, id: Uuid) {
        fn find_and_clone(node: &SplitNode, id: Uuid) -> Option<SplitPane> {
            match node {
                SplitNode::Leaf(p) if p.id == id => Some(p.clone()),
                SplitNode::Leaf(_) => None,
                SplitNode::Container { children, .. } => {
                    children.iter().find_map(|c| find_and_clone(c, id))
                }
            }
        }
        if let Some(pane) = find_and_clone(self, id) {
            *self = SplitNode::Leaf(pane);
        }
    }
}

impl SplitPane {
    /// Create a new SplitPane with the given content, a fresh UUID, and default weight.
    pub fn new_with_content(content: PaneContent) -> Self {
        Self {
            id: Uuid::new_v4(),
            weight: 1.0,
            cwd: PathBuf::from("/"),
            content,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
}

/// A single entry in a directory listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// A listing of active processes within a sector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityListing {
    pub processes: Vec<ProcessEntry>,
}

/// A single process entry with resource usage and optional thumbnail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub mem_usage: u64,
    /// Base64-encoded low-resolution frame buffer thumbnail.
    pub snapshot: Option<String>,
}

/// A search result matching across sectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source_sector: String,
    pub matches: Vec<String>,
}

/// A single line of terminal output with priority and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLine {
    pub text: String,
    /// Line-level priority (1 = Low, 3 = High).
    pub priority: u8,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Persistent settings store with cascading resolution.
///
/// Settings cascade: Application → Sector → Global. The first match wins.
/// Persisted by the Settings Daemon to `~/.config/tos/settings.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsStore {
    #[serde(default)]
    pub global: HashMap<String, String>,
    #[serde(default)]
    pub sectors: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub applications: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub ai_patterns: HashMap<String, String>,
    /// Secure settings store for sensitive credentials (API keys, tokens).
    /// These are NEVER serialized to the Face.
    #[serde(skip_serializing)]
    #[serde(default)]
    pub secure: HashMap<String, String>,
}


impl SettingsStore {
    /// Cascading resolution engine: Application → Sector → Global.
    ///
    /// Returns the most specific value for a given key, falling through
    /// from application scope to sector scope to global scope.
    pub fn resolve(
        &self,
        key: &str,
        sector_id: Option<&str>,
        app_id: Option<&str>,
    ) -> Option<String> {
        if let Some(app) = app_id {
            if let Some(app_settings) = self.applications.get(app) {
                if let Some(val) = app_settings.get(key) {
                    return Some(val.clone());
                }
            }
        }
        if let Some(sec) = sector_id {
            if let Some(sec_settings) = self.sectors.get(sec) {
                if let Some(val) = sec_settings.get(key) {
                    return Some(val.clone());
                }
            }
        }
        self.global.get(key).cloned()
    }
}

/// The system-wide state of the Brain core logic process.
///
/// Serialized in full over the WebSocket state sync channel. The Face
/// renders exclusively from this struct — it has no local state of its own.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TosState {
    pub current_level: HierarchyLevel,
    pub sectors: Vec<Sector>,
    pub active_sector_index: usize,
    pub settings: SettingsStore,
    pub privacy_active: bool,
    pub timeline_cursor: Option<usize>,
    pub timeline_history_len: usize,
    pub pending_confirmation: Option<ConfirmationRequest>,
    pub system_log: Vec<TerminalLine>,
    pub sys_prefix: String,
    pub sys_title: String,
    pub sys_status: String,
    pub brain_time: String,
    pub active_terminal_module: String,
    pub available_modules: Vec<TerminalOutputModuleMeta>,
    pub active_shell_module: String,
    pub available_shell_modules: Vec<ShellModuleMeta>,
    pub active_ai_module: String,
    pub available_ai_modules: Vec<AiModuleMetadata>,
    /// Registered AI behavior modules and their per-behavior configurations.
    pub ai_behaviors: Vec<AiBehavior>,
    pub bezel_expanded: bool,
    /// System-wide default AI backend module ID (cascade base).
    pub ai_default_backend: String,
    pub active_theme: String,
    pub available_themes: Vec<ThemeModule>,
    pub device_profile: crate::ipc::FaceProfile,
    pub ai_offline_queue: Vec<QueuedAiRequest>,
    /// Currently active agents and their task associations.
    pub active_agents: Vec<AgentState>,
    /// Prioritized list of active agents for Agent Stacking instruction merging.
    pub active_agent_stack: Vec<String>,
    /// List of active curators whose context will be aggregated for AI queries.
    pub active_curators: Vec<String>,
    pub active_bezel_components: Vec<BezelComponentState>,
    pub accessibility: AccessibilityState,
    pub version: u64,
}

impl Default for TosState {
    fn default() -> Self {
        let sector = Sector {
            id: Uuid::new_v4(),
            name: "Primary".to_string(),
            hubs: vec![CommandHub {
                id: Uuid::new_v4(),
                mode: CommandHubMode::Command,
                prompt: String::new(),
                current_directory: PathBuf::from("/"),
                terminal_output: vec![],
                buffer_limit: 500,
                shell_listing: None,
                activity_listing: None,
                search_results: None,
                staged_command: None,
                ai_explanation: None,
                json_context: None,
                shell_module: Some("tos-shell-fish".to_string()),
                split_layout: None,
                focused_pane_id: None,
                version: 0,
                ai_history: vec![],
                active_thoughts: vec![],
                last_exit_status: None,
                is_running: false,
            }],
            active_hub_index: 0,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: TrustTier::System,
            priority: 1,
            active_apps: vec![],
            active_app_index: 0,
            participants: vec![],
            kanban_board: None,
            version: 0,
        };

        Self {
            current_level: HierarchyLevel::GlobalOverview,
            sectors: vec![sector],
            active_sector_index: 0,
            settings: SettingsStore::default(),
            privacy_active: false,
            timeline_cursor: None,
            timeline_history_len: 0,
            pending_confirmation: None,
            system_log: vec![],
            sys_prefix: "TOS // SYSTEM-BRAIN".to_string(),
            sys_title: "TOS CORE BRAIN".to_string(),
            sys_status: "SYSTEM READY".to_string(),
            brain_time: "00:00:00".to_string(),
            active_terminal_module: "tos-terminal-lcars".to_string(),
            available_modules: vec![
                TerminalOutputModuleMeta {
                    id: "tos-terminal-lcars".to_string(),
                    name: "LCARS Rectangular".to_string(),
                    version: "1.0.0".to_string(),
                    layout: TerminalLayoutType::Rectangular,
                    supports_high_contrast: true,
                    supports_reduced_motion: true,
                },
                TerminalOutputModuleMeta {
                    id: "tos-terminal-cinematic".to_string(),
                    name: "Cinematic".to_string(),
                    version: "1.0.0".to_string(),
                    layout: TerminalLayoutType::Cinematic,
                    supports_high_contrast: false,
                    supports_reduced_motion: false,
                },
            ],
            active_shell_module: "tos-shell-fish".to_string(),
            available_shell_modules: vec![],
            active_ai_module: "tos-cortex-pro".to_string(),
            available_ai_modules: vec![
                AiModuleMetadata {
                    id: "tos-cortex-pro".to_string(),
                    name: "Cortex Pro".to_string(),
                    version: "1.0.0".to_string(),
                    author: "TOS".to_string(),
                    capabilities: vec!["text".to_string()],
                }
            ],
            ai_behaviors: vec![],
            bezel_expanded: false,
            ai_default_backend: "tos-cortex-pro".to_string(),
            active_theme: "tos-theme-obsidian".to_string(),
            available_themes: vec![
                ThemeModule {
                    id: "tos-theme-obsidian".to_string(),
                    name: "Obsidian".to_string(),
                    version: "1.0.0".to_string(),
                    author: "TOS".to_string(),
                    description: "Dark".to_string(),
                    assets: ThemeAssetDefinition {
                        css: "".to_string(),
                        fonts: vec![],
                        icons: "".to_string(),
                    },
                },
                ThemeModule {
                    id: "tos-theme-light".to_string(),
                    name: "Light".to_string(),
                    version: "1.0.0".to_string(),
                    author: "TOS".to_string(),
                    description: "Light".to_string(),
                    assets: ThemeAssetDefinition {
                        css: "".to_string(),
                        fonts: vec![],
                        icons: "".to_string(),
                    },
                },
                ThemeModule {
                    id: "tos-theme-hc".to_string(),
                    name: "High Contrast".to_string(),
                    version: "1.0.0".to_string(),
                    author: "TOS".to_string(),
                    description: "HC".to_string(),
                    assets: ThemeAssetDefinition {
                        css: "".to_string(),
                        fonts: vec![],
                        icons: "".to_string(),
                    },
                },
            ],
            device_profile: crate::ipc::FaceProfile::Desktop,
            ai_offline_queue: vec![],
            active_agents: vec![],
            active_agent_stack: vec![],
            active_curators: vec![],
            active_bezel_components: vec![],
            accessibility: AccessibilityState::default(),
            version: 0,
        }
    }
}
