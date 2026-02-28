use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

/// TOC §5: The Extended Hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchyLevel {
    GlobalOverview = 1,
    CommandHub = 2,
    ApplicationFocus = 3,
    DetailView = 4,
    BufferView = 5,
}

/// TOC §7.3: Four Augmentation Modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandHubMode {
    Command,
    Directory,
    Activity,
    Search,
    Ai,
}

/// §18: Dual-Tier Trust Model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustTier {
    Standard, // Sandboxed (§18.4)
    System,   // Privileged (§18.7)
}

/// §17.3: Dangerous Command Handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    pub id: Uuid,
    pub original_request: String,
    pub message: String,
    pub progress: f32, // 0.0 to 1.0 for tactile slider
}
/// TOC §10: Sectors and the Tree Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: Uuid,
    pub name: String,
    pub hubs: Vec<CommandHub>,
    pub active_hub_index: usize,
    pub frozen: bool, // §6.5: Freeze stops UI updates
    pub is_remote: bool, // §12: Remote status
    pub disconnected: bool, // §27.3: Connection status
    pub trust_tier: TrustTier, // §18
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHub {
    pub id: Uuid,
    pub mode: CommandHubMode,
    pub prompt: String,
    pub current_directory: PathBuf,
    pub terminal_output: Vec<TerminalLine>,
    pub buffer_limit: usize,
    pub shell_listing: Option<DirectoryListing>, // §27.3: Local/Remote directory data
    pub activity_listing: Option<ActivityListing>, // §7.3: Activity mode data
    pub search_results: Option<Vec<SearchResult>>, // §7.3: Search mode results
    pub staged_command: Option<String>,           // §12: AI-proposed command
    pub ai_explanation: Option<String>,           // §12: AI-side documentation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityListing {
    pub processes: Vec<ProcessEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub mem_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source_sector: String,
    pub matches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLine {
    pub text: String,
    pub priority: u8, // §27.5: Line-Level Priority
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// TOC §3.1: The Brain (Logic Process) State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TosState {
    pub current_level: HierarchyLevel,
    pub sectors: Vec<Sector>,
    pub active_sector_index: usize,
    pub settings: std::collections::HashMap<String, String>,
    pub pending_confirmation: Option<ConfirmationRequest>, // §17.3
    pub system_log: Vec<TerminalLine>, // §6.2, §19.1
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
                current_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
                terminal_output: vec![],
                buffer_limit: 500, // §29.2 default
                shell_listing: None,
                activity_listing: None,
                search_results: None,
                staged_command: None,
                ai_explanation: None,
            }],
            active_hub_index: 0,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: TrustTier::System,
        };

        Self {
            current_level: HierarchyLevel::GlobalOverview,
            sectors: vec![sector],
            active_sector_index: 0,
            settings: std::collections::HashMap::new(),
            pending_confirmation: None,
            system_log: vec![],
        }
    }
}
