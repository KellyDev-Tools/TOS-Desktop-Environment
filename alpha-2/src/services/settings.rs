use std::io::{Write, BufRead, BufReader};
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
            let _ = stream.write_all(format!("save:{}\n", json).as_bytes());
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
        Self::build_default_settings()
    }

    /// Public accessor for tests to validate the default settings schema.
    pub fn default_settings_public(&self) -> SettingsStore {
        Self::build_default_settings()
    }

    /// Constructs the canonical default settings with all Alpha-2.2 namespaces.
    fn build_default_settings() -> SettingsStore {
        let mut map = HashMap::new();

        // --- Core Settings (Alpha-2.1 legacy) ---
        map.insert("theme".to_string(), "lcars-light".to_string());
        map.insert("default_shell".to_string(), "fish".to_string());
        map.insert("terminal_output_module".to_string(), "rectangular".to_string());
        map.insert("master_volume".to_string(), "80".to_string());
        map.insert("logging_enabled".to_string(), "true".to_string());
        map.insert("terminal_buffer_limit".to_string(), "500".to_string());

        // --- Onboarding (Onboarding Specification §2) ---
        map.insert("tos.onboarding.first_run_complete".to_string(), "false".to_string());
        map.insert("tos.onboarding.wizard_complete".to_string(), "false".to_string());
        map.insert("tos.onboarding.hint_suppressed".to_string(), "false".to_string());
        map.insert("tos.onboarding.sessions_count".to_string(), "0".to_string());
        map.insert("tos.onboarding.commands_run".to_string(), "0".to_string());

        // --- Trust (Trust & Confirmation Specification §2, §6) ---
        // Both command classes default to WARN — the user explicitly promotes.
        map.insert("tos.trust.privilege_escalation".to_string(), "warn".to_string());
        map.insert("tos.trust.recursive_bulk".to_string(), "warn".to_string());
        map.insert("tos.trust.bulk_threshold".to_string(), "10".to_string());

        // --- AI (AI Co-Pilot Specification §9) ---
        map.insert("tos.ai.default_backend".to_string(), "tos-ai-standard".to_string());
        map.insert("tos.ai.chip_color".to_string(), "secondary".to_string());
        map.insert("tos.ai.ghost_text_opacity".to_string(), "40".to_string());
        map.insert("tos.ai.disabled".to_string(), "false".to_string());
        map.insert("tos.ai.context_level".to_string(), "standard".to_string());

        // --- Expanded Bezel (Expanded Bezel Specification §7) ---
        map.insert("tos.interface.bezel.dismiss_behavior".to_string(), "stay_open".to_string());
        map.insert("tos.interface.bezel.auto_collapse_timeout".to_string(), "5".to_string());

        // --- Split Viewport (Split Viewport Specification §6) ---
        map.insert("tos.interface.splits.divider_snap".to_string(), "true".to_string());

        // --- Network (Ecosystem Orchestration / Anchor Port) ---
        map.insert("tos.network.anchor_port".to_string(), "7000".to_string());
        map.insert("tos.network.mdns_enabled".to_string(), "true".to_string());
        map.insert("tos.network.remote_access".to_string(), "false".to_string());

        SettingsStore {
            global: map,
            sectors: HashMap::new(),
            applications: HashMap::new(),
        }
    }
}

