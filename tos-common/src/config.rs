//! TOS Brain configuration loader.
//!
//! Reads `tos.toml` from one of these locations (in priority order):
//! 1. Path passed via `--config <path>` CLI argument
//! 2. `TOS_CONFIG` environment variable
//! 3. `~/.config/tos/tos.toml`
//! 4. `./tos.toml` (current working directory)
//! 5. Built-in defaults
//!
//! The config is divided into three conceptual layers:
//! - **Platform:** Filesystem base paths that vary per target (Linux, Android, XR).
//! - **Local:** Direct-persistence settings for single-machine operation.
//! - **System:** Low-level OS integration (crash reporting, telemetry).
//! - **Remote:** Network binding, daemon delegation, and multi-Face coordination.

use std::path::PathBuf;

// ──────────────────────────────────────────────────────────────────────────
// Platform — filesystem layout
// ──────────────────────────────────────────────────────────────────────────

/// Platform-specific base directories.
/// Empty strings mean "auto-detect from the runtime environment".
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct PlatformConfig {
    /// Base config dir (settings, tos.toml copy).
    #[serde(default)]
    pub config_dir: String,
    /// Base data dir (sessions, logs, marketplace cache).
    #[serde(default)]
    pub data_dir: String,
    /// Base runtime dir (PID files, sockets).
    #[serde(default)]
    pub runtime_dir: String,
}


impl PlatformConfig {
    /// Resolve config_dir to an absolute path.
    /// Falls back to platform-detected XDG/home dir.
    pub fn resolved_config_dir(&self) -> PathBuf {
        if !self.config_dir.is_empty() {
            return PathBuf::from(&self.config_dir);
        }
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("tos")
    }

    /// Resolve data_dir to an absolute path.
    /// Falls back to platform-detected XDG local data dir.
    pub fn resolved_data_dir(&self) -> PathBuf {
        if !self.data_dir.is_empty() {
            return PathBuf::from(&self.data_dir);
        }
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("tos")
    }

    /// Resolve runtime_dir to an absolute path.
    /// Falls back to $XDG_RUNTIME_DIR/tos or /tmp/tos.
    pub fn resolved_runtime_dir(&self) -> PathBuf {
        if !self.runtime_dir.is_empty() {
            return PathBuf::from(&self.runtime_dir);
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(xdg).join("tos")
        } else {
            PathBuf::from("/tmp/tos")
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Local — direct disk I/O
// ──────────────────────────────────────────────────────────────────────────

/// Controls how the Brain handles persistence when operating standalone
/// (no daemon delegation).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LocalConfig {
    /// If true, Brain writes settings/sessions to disk itself.
    /// If false, requires tos-settingsd / tos-sessiond daemons.
    #[serde(default = "default_true")]
    pub persistence: bool,
    /// Active working directory. Empty = cwd at startup.
    #[serde(default)]
    pub active_dir: String,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            persistence: true,
            active_dir: String::new(),
        }
    }
}

impl LocalConfig {
    pub fn resolved_active_dir(&self) -> PathBuf {
        if !self.active_dir.is_empty() {
            PathBuf::from(&self.active_dir)
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Remote — network & daemon delegation
// ──────────────────────────────────────────────────────────────────────────

/// Controls network binding and remote Face access.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RemoteConfig {
    /// TCP command server port.
    #[serde(default = "default_anchor_port")]
    pub anchor_port: u16,
    /// WebSocket state-sync port.
    #[serde(default = "default_ws_port")]
    pub ws_port: u16,
    /// Address to bind listeners to.
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    /// Unix domain socket path for local IPC.
    #[serde(default = "default_uds_path")]
    pub uds_path: String,
    /// Advertise via mDNS.
    #[serde(default = "default_true")]
    pub mdns_enabled: bool,
    /// Allow connections from non-localhost.
    #[serde(default)]
    pub remote_access: bool,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            anchor_port: 7000,
            ws_port: 7001,
            bind_address: "0.0.0.0".to_string(),
            uds_path: "/tmp/brain.sock".to_string(),
            mdns_enabled: true,
            remote_access: false,
        }
    }
}

fn default_anchor_port() -> u16 {
    7000
}
fn default_ws_port() -> u16 {
    7001
}
fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}
fn default_uds_path() -> String {
    "/tmp/brain.sock".to_string()
}
fn default_true() -> bool {
    true
}

// ──────────────────────────────────────────────────────────────────────────
// Session — session persistence specifics
// ──────────────────────────────────────────────────────────────────────────

/// Session persistence configuration.
/// Inherits `local.persistence` for the local-vs-daemon decision.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionConfig {
    /// Directory for session files. Empty = {data_dir}/sessions/.
    #[serde(default)]
    pub sessions_dir: String,
    /// Debounce interval (ms) for live auto-saves.
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

fn default_debounce_ms() -> u64 {
    2000
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            sessions_dir: String::new(),
            debounce_ms: 2000,
        }
    }
}

impl SessionConfig {
    /// Resolve sessions_dir using the platform data_dir as the base if empty.
    pub fn resolved_sessions_dir(&self, platform: &PlatformConfig) -> PathBuf {
        if !self.sessions_dir.is_empty() {
            PathBuf::from(&self.sessions_dir)
        } else {
            platform.resolved_data_dir().join("sessions")
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Settings — settings persistence specifics
// ──────────────────────────────────────────────────────────────────────────

/// Settings persistence configuration.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SettingsConfig {
    /// Path to the settings JSON file. Empty = {config_dir}/settings.json.
    #[serde(default)]
    pub settings_path: String,
    /// Port for the optional tos-settingsd daemon.
    #[serde(default = "default_settings_daemon_port")]
    pub daemon_port: u16,
}

fn default_settings_daemon_port() -> u16 {
    7002
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            settings_path: String::new(),
            daemon_port: 7002,
        }
    }
}

impl SettingsConfig {
    /// Resolve settings_path using the platform config_dir as the base if empty.
    pub fn resolved_settings_path(&self, platform: &PlatformConfig) -> PathBuf {
        if !self.settings_path.is_empty() {
            PathBuf::from(&self.settings_path)
        } else {
            platform.resolved_config_dir().join("settings.json")
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// System — OS integration
// ──────────────────────────────────────────────────────────────────────────
 
/// Low-level system integration settings.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SystemConfig {
    /// If true, panics generate automated crash reports sent to the Brain.
    #[serde(default = "default_false")]
    pub crash_reporting_enabled: bool,
}

fn default_false() -> bool {
    false
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            crash_reporting_enabled: false,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Brain — identity of this Brain instance
// ──────────────────────────────────────────────────────────────────────────

/// Identifies this Brain in a multi-brain mesh.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BrainConfig {
    /// Human-readable name. Empty = system hostname.
    #[serde(default)]
    pub name: String,
    /// UUID for this instance. Empty = auto-generated on first boot.
    #[serde(default)]
    pub id: String,
    /// Role: "standalone", "primary", or "satellite".
    #[serde(default = "default_brain_role")]
    pub role: String,
    /// Upstream Brain address (host:port) for satellite mode.
    #[serde(default)]
    pub upstream: String,
}

fn default_brain_role() -> String {
    "standalone".to_string()
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: String::new(),
            role: "standalone".to_string(),
            upstream: String::new(),
        }
    }
}

impl BrainConfig {
    /// Resolve the Brain name. Falls back to system hostname.
    pub fn resolved_name(&self) -> String {
        if !self.name.is_empty() {
            return self.name.clone();
        }
        // Read hostname from /etc/hostname (Linux/XR) or fall back.
        std::fs::read_to_string("/etc/hostname")
            .map(|h| h.trim().to_string())
            .unwrap_or_else(|_| "TOS-Brain".to_string())
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Face — how Faces discover and connect
// ──────────────────────────────────────────────────────────────────────────

/// Controls Face connection policy and advertisement.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FaceConfig {
    /// Max concurrent Face connections. 0 = unlimited.
    #[serde(default)]
    pub max_connections: u32,
    /// Preferred URL advertised via mDNS. Empty = auto-detect.
    #[serde(default)]
    pub advertised_url: String,
    /// Serve the built-in Svelte web Face.
    #[serde(default = "default_true")]
    pub serve_web_face: bool,
    /// Port for the embedded web Face dev server.
    #[serde(default = "default_web_face_port")]
    pub web_face_port: u16,
}

fn default_web_face_port() -> u16 {
    5173
}

impl Default for FaceConfig {
    fn default() -> Self {
        Self {
            max_connections: 0,
            advertised_url: String::new(),
            serve_web_face: true,
            web_face_port: 5173,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// TosConfig — the top-level config object
// ──────────────────────────────────────────────────────────────────────────

/// Top-level configuration, deserialized from `tos.toml`.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct TosConfig {
    #[serde(default)]
    pub brain: BrainConfig,
    #[serde(default)]
    pub face: FaceConfig,
    #[serde(default)]
    pub platform: PlatformConfig,
    #[serde(default)]
    pub local: LocalConfig,
    #[serde(default)]
    pub remote: RemoteConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub settings: SettingsConfig,
    #[serde(default)]
    pub system: SystemConfig,
}


impl TosConfig {
    /// Load config from the first available source.
    pub fn load() -> Self {
        Self::load_from(None)
    }

    /// Load config, optionally overriding the search path with a CLI-provided path.
    pub fn load_from(cli_path: Option<&str>) -> Self {
        let candidates = Self::candidate_paths(cli_path);

        for path in &candidates {
            if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(content) => match toml::from_str::<TosConfig>(&content) {
                        Ok(cfg) => {
                            tracing::info!("Loaded TOS config from {:?}", path);
                            return cfg;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to read {:?}: {}", path, e);
                    }
                }
            }
        }

        tracing::info!("No tos.toml found; using built-in defaults");
        Self::default()
    }

    /// Build the ordered list of candidate config file paths.
    fn candidate_paths(cli_path: Option<&str>) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. CLI override.
        if let Some(p) = cli_path {
            paths.push(PathBuf::from(p));
        }

        // 2. Environment variable.
        if let Ok(env_path) = std::env::var("TOS_CONFIG") {
            paths.push(PathBuf::from(env_path));
        }

        // 3. XDG config dir (~/.config/tos/tos.toml).
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("tos/tos.toml"));
        }

        // 4. Current working directory.
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("tos.toml"));
        }

        paths
    }

    // ── Convenience resolvers ──────────────────────────────────────────

    /// Resolved session directory path.
    pub fn sessions_dir(&self) -> PathBuf {
        self.session.resolved_sessions_dir(&self.platform)
    }

    /// Resolved settings file path.
    pub fn settings_path(&self) -> PathBuf {
        self.settings.resolved_settings_path(&self.platform)
    }

    /// Resolved active working directory.
    pub fn active_dir(&self) -> PathBuf {
        self.local.resolved_active_dir()
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = TosConfig::default();
        assert!(cfg.local.persistence);
        assert_eq!(cfg.session.debounce_ms, 2000);
        assert_eq!(cfg.remote.anchor_port, 7000);
        assert_eq!(cfg.remote.ws_port, 7001);
        assert_eq!(cfg.settings.daemon_port, 7002);
        assert!(!cfg.remote.remote_access);
        assert!(cfg.remote.mdns_enabled);
    }

    #[test]
    fn test_parse_full_toml() {
        let toml_str = r#"
[platform]
config_dir = "/etc/tos"
data_dir = "/var/lib/tos"
runtime_dir = "/run/tos"

[local]
persistence = false
active_dir = "/home/user/projects"

[remote]
anchor_port = 8000
ws_port = 8001
bind_address = "127.0.0.1"
uds_path = "/run/tos/brain.sock"
mdns_enabled = false
remote_access = true

[session]
sessions_dir = "/var/lib/tos/sessions"
debounce_ms = 500

[settings]
settings_path = "/etc/tos/settings.json"
daemon_port = 9002
"#;
        let cfg: TosConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.platform.config_dir, "/etc/tos");
        assert_eq!(cfg.platform.data_dir, "/var/lib/tos");
        assert_eq!(cfg.platform.runtime_dir, "/run/tos");
        assert!(!cfg.local.persistence);
        assert_eq!(cfg.local.active_dir, "/home/user/projects");
        assert_eq!(cfg.remote.anchor_port, 8000);
        assert_eq!(cfg.remote.ws_port, 8001);
        assert_eq!(cfg.remote.bind_address, "127.0.0.1");
        assert!(cfg.remote.remote_access);
        assert!(!cfg.remote.mdns_enabled);
        assert_eq!(cfg.session.debounce_ms, 500);
        assert_eq!(cfg.settings.daemon_port, 9002);
    }

    #[test]
    fn test_partial_toml_uses_defaults() {
        let toml_str = r#"
[session]
debounce_ms = 1000
"#;
        let cfg: TosConfig = toml::from_str(toml_str).unwrap();
        // Explicitly set:
        assert_eq!(cfg.session.debounce_ms, 1000);
        // Everything else should be default:
        assert!(cfg.local.persistence);
        assert_eq!(cfg.remote.anchor_port, 7000);
        assert!(cfg.session.sessions_dir.is_empty());
    }

    #[test]
    fn test_platform_resolution_custom() {
        let platform = PlatformConfig {
            config_dir: "/custom/config".to_string(),
            data_dir: "/custom/data".to_string(),
            runtime_dir: "/custom/runtime".to_string(),
        };
        assert_eq!(
            platform.resolved_config_dir(),
            PathBuf::from("/custom/config")
        );
        assert_eq!(platform.resolved_data_dir(), PathBuf::from("/custom/data"));
        assert_eq!(
            platform.resolved_runtime_dir(),
            PathBuf::from("/custom/runtime")
        );
    }

    #[test]
    fn test_platform_resolution_defaults() {
        let platform = PlatformConfig::default();
        let config = platform.resolved_config_dir();
        let data = platform.resolved_data_dir();
        // Should resolve to real XDG paths containing "tos"
        assert!(config.to_string_lossy().contains("tos"));
        assert!(data.to_string_lossy().contains("tos"));
    }

    #[test]
    fn test_sessions_dir_inherits_platform() {
        let platform = PlatformConfig {
            data_dir: "/srv/tos-data".to_string(),
            ..Default::default()
        };
        let session = SessionConfig::default();
        assert_eq!(
            session.resolved_sessions_dir(&platform),
            PathBuf::from("/srv/tos-data/sessions")
        );
    }

    #[test]
    fn test_sessions_dir_custom_overrides_platform() {
        let platform = PlatformConfig {
            data_dir: "/srv/tos-data".to_string(),
            ..Default::default()
        };
        let session = SessionConfig {
            sessions_dir: "/explicit/sessions".to_string(),
            ..Default::default()
        };
        assert_eq!(
            session.resolved_sessions_dir(&platform),
            PathBuf::from("/explicit/sessions")
        );
    }

    #[test]
    fn test_settings_path_inherits_platform() {
        let platform = PlatformConfig {
            config_dir: "/etc/tos".to_string(),
            ..Default::default()
        };
        let settings = SettingsConfig::default();
        assert_eq!(
            settings.resolved_settings_path(&platform),
            PathBuf::from("/etc/tos/settings.json")
        );
    }

    #[test]
    fn test_convenience_resolvers() {
        let cfg = TosConfig {
            platform: PlatformConfig {
                data_dir: "/d".to_string(),
                config_dir: "/c".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(cfg.sessions_dir(), PathBuf::from("/d/sessions"));
        assert_eq!(cfg.settings_path(), PathBuf::from("/c/settings.json"));
    }
}
