use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub module_type: String, // "Application", "TerminalOutput", "Theme", etc.
    pub author: String,
}

pub struct MarketplaceService;

impl MarketplaceService {
    /// §18.1: Discover module in a directory
    pub fn discover_module(mut path: PathBuf) -> anyhow::Result<ModuleManifest> {
        path.push("module.toml");
        
        let content = std::fs::read_to_string(path)?;
        let manifest: ModuleManifest = toml::from_str(&content)?;
        Ok(manifest)
    }
}
