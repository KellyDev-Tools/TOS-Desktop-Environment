use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

/// The system hierarchy levels defining the visual depth of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchyLevel {
    GlobalOverview = 1,
    CommandHub = 2,
    ApplicationFocus = 3,
    DetailView = 4,
    BufferView = 5,
}

/// The operational augmentation modes for the Command Hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandHubMode {
    Command,
    Directory,
    Activity,
    Search,
    Ai,
}

/// Defines the security trust level for operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustTier {
    Standard, // Sandboxed
    System,   // Privileged
}

/// Security validation for execution of dangerous system commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    pub id: Uuid,
    pub original_request: String,
    pub message: String,
    pub progress: f32, // 0.0 to 1.0 for tactile slider
}
/// Sectors and the hierarchical tree model structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: Uuid,
    pub name: String,
    pub hubs: Vec<CommandHub>,
    pub active_hub_index: usize,
    pub frozen: bool, // Freeze stops UI updates for this sector
    pub is_remote: bool, // Remote vs Local status
    pub disconnected: bool, // Connection status for remote sectors
    pub trust_tier: TrustTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHub {
    pub id: Uuid,
    pub mode: CommandHubMode,
    pub prompt: String,
    pub current_directory: PathBuf,
    pub terminal_output: Vec<TerminalLine>,
    pub buffer_limit: usize,
    pub shell_listing: Option<DirectoryListing>, // Local or Remote directory data
    pub activity_listing: Option<ActivityListing>, // Activity mode process data
    pub search_results: Option<Vec<SearchResult>>, // Search mode matches
    pub staged_command: Option<String>,           // AI-proposed command for review
    pub ai_explanation: Option<String>,           // AI rationale/documentation
    pub json_context: Option<serde_json::Value>,  // Custom JSON exported via OSC 9004
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
    pub snapshot: Option<String>, // Base64 low-res frame buffer thumbnail
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source_sector: String,
    pub matches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLine {
    pub text: String,
    pub priority: u8, // Line-Level Priority (1=Low, 3=High)
    pub timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsStore {
    pub global: std::collections::HashMap<String, String>,
    pub sectors: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    pub applications: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self {
            global: std::collections::HashMap::new(),
            sectors: std::collections::HashMap::new(),
            applications: std::collections::HashMap::new(),
        }
    }
}

impl SettingsStore {
    /// Cascading resolution engine: Application -> Sector -> Global
    pub fn resolve(&self, key: &str, sector_id: Option<&str>, app_id: Option<&str>) -> Option<String> {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TosState {
    pub current_level: HierarchyLevel,
    pub sectors: Vec<Sector>,
    pub active_sector_index: usize,
    pub settings: SettingsStore,
    pub pending_confirmation: Option<ConfirmationRequest>,
    pub system_log: Vec<TerminalLine>,
    pub sys_prefix: String,
    pub sys_title: String,
    pub sys_status: String,
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
                buffer_limit: 500, // FIFO buffer limit
                shell_listing: None,
                activity_listing: None,
                search_results: None,
                staged_command: None,
                ai_explanation: None,
                json_context: None,
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
            settings: SettingsStore::default(),
            pending_confirmation: None,
            system_log: vec![],
            sys_prefix: "TOS // SYSTEM-BRAIN".to_string(),
            sys_title: "ALPHA-2.1 // INTEL-DRIVEN".to_string(),
            sys_status: "BRAIN: ACTIVE".to_string(),
        }
    }
}
pub mod ipc_dispatcher;
