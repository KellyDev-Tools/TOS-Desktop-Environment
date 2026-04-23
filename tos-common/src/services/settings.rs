use crate::SettingsStore;
use crate::config::TosConfig;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct SettingsService {
    /// Resolved absolute path to settings.json.
    config_path: PathBuf,
    /// Optional registry for dynamic daemon discovery (§4.1).
    registry: Option<Arc<Mutex<crate::services::registry::ServiceRegistry>>>,
    /// Whether to use local disk I/O when daemon is unavailable.
    local_persistence: bool,
}

impl SettingsService {
    /// Construct with default config (platform-detected paths).
    pub fn new() -> Self {
        Self::with_config(&TosConfig::default())
    }

    /// Construct from a full TosConfig, resolving paths through the platform layer.
    pub fn with_config(config: &TosConfig) -> Self {
        Self {
            config_path: config.settings_path(),
            registry: None,
            local_persistence: config.local.persistence,
        }
    }

    pub fn with_registry_and_config(
        registry: Arc<Mutex<crate::services::registry::ServiceRegistry>>,
        config: &TosConfig,
    ) -> Self {
        Self {
            config_path: config.settings_path(),
            registry: Some(registry),
            local_persistence: config.local.persistence,
        }
    }

    /// The resolved settings file path.
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    // ── Remote persistence (tos-settingsd daemon) ────────────────────

    fn save_daemon(&self, settings: &SettingsStore) -> anyhow::Result<()> {
        let port = self
            .registry
            .as_ref()
            .and_then(
                |r: &Arc<Mutex<crate::services::registry::ServiceRegistry>>| {
                    r.lock().unwrap().port_of("tos-settingsd")
                },
            )
            .unwrap_or(7002);

        let addr = format!("127.0.0.1:{}", port);
        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(100),
        )?;
        let json = serde_json::to_string(settings)?;
        stream.write_all(format!("save:{}\n", json).as_bytes())?;
        Ok(())
    }

    fn load_daemon(&self) -> anyhow::Result<SettingsStore> {
        let port = self
            .registry
            .as_ref()
            .and_then(
                |r: &Arc<Mutex<crate::services::registry::ServiceRegistry>>| {
                    r.lock().unwrap().port_of("tos-settingsd")
                },
            )
            .unwrap_or(7002);

        let addr = format!("127.0.0.1:{}", port);
        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(100),
        )?;
        stream.write_all(b"get_all\n")?;
        let mut reader = BufReader::new(&stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        let settings: SettingsStore = serde_json::from_str(&response)?;
        Ok(settings)
    }

    // ── Local persistence (direct disk I/O) ──────────────────────────

    fn save_local(&self, settings: &SettingsStore) -> anyhow::Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.config_path, json)?;
        Ok(())
    }

    pub fn load_local(&self) -> anyhow::Result<SettingsStore> {
        if !self.config_path.exists() {
            return Ok(self.default_settings());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let settings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    // ── Public API (routes to daemon or local) ───────────────────────

    /// Save the current settings. Tries daemon first, falls back to local.
    pub fn save(&self, settings: &SettingsStore) -> anyhow::Result<()> {
        // Try daemon first.
        if self.save_daemon(settings).is_ok() {
            return Ok(());
        }
        // Local fallback.
        if self.local_persistence {
            return self.save_local(settings);
        }
        Err(anyhow::anyhow!(
            "tos-settingsd unavailable and local.persistence is disabled"
        ))
    }

    /// Load settings. Tries daemon first, falls back to local disk.
    pub fn load(&self) -> anyhow::Result<SettingsStore> {
        // Try daemon first.
        if let Ok(settings) = self.load_daemon() {
            return Ok(settings);
        }
        // Local fallback.
        if self.local_persistence {
            return self.load_local();
        }
        // Return defaults as last resort.
        Ok(self.default_settings())
    }

    /// Directly load from disk, bypassing daemon check.
    pub fn load_local_only(&self) -> anyhow::Result<SettingsStore> {
        self.load_local()
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
        map.insert(
            "terminal_output_module".to_string(),
            "rectangular".to_string(),
        );
        map.insert("master_volume".to_string(), "80".to_string());
        map.insert("logging_enabled".to_string(), "true".to_string());
        map.insert("terminal_buffer_limit".to_string(), "500".to_string());

        // --- Onboarding (Onboarding Specification §2) ---
        map.insert(
            "tos.onboarding.first_run_complete".to_string(),
            "false".to_string(),
        );
        map.insert(
            "tos.onboarding.wizard_complete".to_string(),
            "false".to_string(),
        );
        map.insert(
            "tos.onboarding.hint_suppressed".to_string(),
            "false".to_string(),
        );
        map.insert("tos.onboarding.sessions_count".to_string(), "0".to_string());
        map.insert("tos.onboarding.commands_run".to_string(), "0".to_string());

        // --- Trust (Trust & Confirmation Specification §2, §6) ---
        map.insert(
            "tos.trust.privilege_escalation".to_string(),
            "warn".to_string(),
        );
        map.insert("tos.trust.recursive_bulk".to_string(), "warn".to_string());
        map.insert("tos.trust.bulk_threshold".to_string(), "10".to_string());

        // --- AI (AI Co-Pilot Specification §9) ---
        map.insert(
            "tos.ai.default_backend".to_string(),
            "tos-ai-standard".to_string(),
        );
        map.insert("tos.ai.chip_color".to_string(), "secondary".to_string());
        map.insert("tos.ai.ghost_text_opacity".to_string(), "40".to_string());
        map.insert("tos.ai.disabled".to_string(), "false".to_string());
        map.insert("tos.ai.context_level".to_string(), "standard".to_string());

        // --- Expanded Bezel (Expanded Bezel Specification §7) ---
        map.insert(
            "tos.interface.bezel.dismiss_behavior".to_string(),
            "stay_open".to_string(),
        );
        map.insert(
            "tos.interface.bezel.auto_collapse_timeout".to_string(),
            "5".to_string(),
        );

        // --- Split Viewport (Split Viewport Specification §6) ---
        map.insert(
            "tos.interface.splits.divider_snap".to_string(),
            "true".to_string(),
        );

        // --- Network (Ecosystem Orchestration / Anchor Port) ---
        map.insert("tos.network.anchor_port".to_string(), "7000".to_string());
        map.insert("tos.network.mdns_enabled".to_string(), "true".to_string());
        map.insert("tos.network.remote_access".to_string(), "false".to_string());

        SettingsStore {
            global: map,
            sectors: HashMap::new(),
            applications: HashMap::new(),
            ai_patterns: HashMap::new(),
        }
    }
}
