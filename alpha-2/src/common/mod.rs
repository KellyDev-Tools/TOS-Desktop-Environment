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

/// TOC §10: Sectors and the Tree Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: Uuid,
    pub name: String,
    pub hubs: Vec<CommandHub>,
    pub active_hub_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHub {
    pub id: Uuid,
    pub mode: CommandHubMode,
    pub prompt: String,
    pub current_directory: PathBuf,
    pub terminal_output: Vec<TerminalLine>,
    pub buffer_limit: usize,
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
            }],
            active_hub_index: 0,
        };

        Self {
            current_level: HierarchyLevel::GlobalOverview,
            sectors: vec![sector],
            active_sector_index: 0,
            settings: std::collections::HashMap::new(),
        }
    }
}
