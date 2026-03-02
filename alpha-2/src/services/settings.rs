use std::collections::HashMap;
use std::path::PathBuf;
// use std::sync::{Arc, Mutex}; // Unused after TosState decoupling
// use crate::common::TosState; // Unused after TosState decoupling

pub struct SettingsService {
    config_path: PathBuf,
}

impl SettingsService {
    pub fn new() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.push(".config/tos/settings.json");
        
        Self {
            config_path: home,
        }
    }

    /// Save the current settings collection to persistent storage.
    pub fn save(&self, settings: &HashMap<String, String>) -> anyhow::Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Load settings from disk
    pub fn load(&self) -> anyhow::Result<HashMap<String, String>> {
        if !self.config_path.exists() {
            return Ok(self.default_settings());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let settings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    fn default_settings(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("theme".to_string(), "lcars-light".to_string());
        map.insert("default_shell".to_string(), "fish".to_string());
        map.insert("terminal_output_module".to_string(), "rectangular".to_string());
        map.insert("master_volume".to_string(), "80".to_string());
        map.insert("logging_enabled".to_string(), "true".to_string());
        map.insert("terminal_buffer_limit".to_string(), "500".to_string());
        map
    }
}
