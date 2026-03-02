use std::io::{Read, Write, BufRead, BufReader};
use std::net::TcpStream;
use std::collections::HashMap;
use crate::common::SettingsStore;

pub struct SettingsService {
    config_path: std::path::PathBuf,
    daemon_addr: String,
}

impl SettingsService {
    pub fn new() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        home.push(".config/tos/settings.json");
        
        Self {
            config_path: home,
            daemon_addr: "127.0.0.1:7002".to_string(),
        }
    }

    /// Save the current settings collection. Prioritizes the Settings Daemon if active.
    pub fn save(&self, settings: &SettingsStore) -> anyhow::Result<()> {
        if let Ok(mut stream) = TcpStream::connect_timeout(&self.daemon_addr.parse().unwrap(), std::time::Duration::from_millis(100)) {
            let json = serde_json::to_string(settings)?;
            let _ = stream.write_all(b"save\n");
            return Ok(());
        }

        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Load settings. Attempts to fetch from the Settings Daemon first, falling back to disk.
    pub fn load(&self) -> anyhow::Result<SettingsStore> {
        if let Ok(mut stream) = TcpStream::connect_timeout(&self.daemon_addr.parse().unwrap(), std::time::Duration::from_millis(100)) {
            let _ = stream.write_all(b"get_all\n");
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if let Ok(_) = reader.read_line(&mut response) {
                if let Ok(settings) = serde_json::from_str(&response) {
                    return Ok(settings);
                }
            }
        }

        if !self.config_path.exists() {
            return Ok(self.default_settings());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let settings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    /// Directly load from disk, bypassing daemon check.
    pub fn load_local(&self) -> anyhow::Result<SettingsStore> {
        if !self.config_path.exists() {
            return Ok(self.default_settings());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let settings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    fn default_settings(&self) -> SettingsStore {
        let mut map = HashMap::new();
        map.insert("theme".to_string(), "lcars-light".to_string());
        map.insert("default_shell".to_string(), "fish".to_string());
        map.insert("terminal_output_module".to_string(), "rectangular".to_string());
        map.insert("master_volume".to_string(), "80".to_string());
        map.insert("logging_enabled".to_string(), "true".to_string());
        map.insert("terminal_buffer_limit".to_string(), "500".to_string());
        
        SettingsStore {
            global: map,
            sectors: HashMap::new(),
            applications: HashMap::new(),
        }
    }
}
