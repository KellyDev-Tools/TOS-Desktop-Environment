use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::services::registry::ServiceRegistry;
use crate::common::TosState;
use crate::config::TosConfig;

#[derive(Clone)]
pub struct SessionService {
    registry: Arc<Mutex<ServiceRegistry>>,
    last_trigger: Arc<Mutex<std::time::Instant>>,
    /// Resolved absolute path to the sessions directory.
    sessions_dir: PathBuf,
    /// Whether to write directly to disk (true) or require tos-sessiond (false).
    local_persistence: bool,
    /// Debounce interval in milliseconds.
    debounce_ms: u64,
}

impl SessionService {
    /// Construct with default config (local-first, platform-default paths).
    pub fn new(registry: Arc<Mutex<ServiceRegistry>>) -> Self {
        Self::with_config(registry, &TosConfig::default())
    }

    /// Construct from a full TosConfig, resolving paths through the platform layer.
    pub fn with_config(registry: Arc<Mutex<ServiceRegistry>>, config: &TosConfig) -> Self {
        let sessions_dir = config.sessions_dir();
        let local_persistence = config.local.persistence;
        let debounce_ms = config.session.debounce_ms;

        // Ensure directory exists on construction.
        if let Err(e) = std::fs::create_dir_all(&sessions_dir) {
            tracing::warn!("SessionService: could not create sessions dir {:?}: {}", sessions_dir, e);
        }

        Self {
            registry,
            last_trigger: Arc::new(Mutex::new(std::time::Instant::now())),
            sessions_dir,
            local_persistence,
            debounce_ms,
        }
    }

    /// The resolved sessions directory.
    pub fn sessions_dir(&self) -> &PathBuf {
        &self.sessions_dir
    }

    fn get_daemon_address(&self) -> Option<String> {
        let reg = self.registry.lock().unwrap();
        reg.port_of("tos-sessiond").map(|port| format!("127.0.0.1:{}", port))
    }

    // ── Local persistence (direct disk I/O) ──────────────────────────

    fn save_live_local(&self, state: &TosState) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.sessions_dir)?;
        let live_path = self.sessions_dir.join("_live.tos-session");
        let tmp_path = self.sessions_dir.join("_live.tos-session.tmp");
        let json = serde_json::to_string(state)?;
        std::fs::write(&tmp_path, &json)?;
        std::fs::rename(&tmp_path, &live_path)?;
        Ok(())
    }

    fn save_named_local(&self, sector_id: &str, name: &str, state: &TosState) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.sessions_dir)?;
        let path = self.sessions_dir.join(format!("{}_{}.tos-session", sector_id, name));
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    fn load_named_local(&self, sector_id: &str, name: &str) -> anyhow::Result<String> {
        let path = self.sessions_dir.join(format!("{}_{}.tos-session", sector_id, name));
        Ok(std::fs::read_to_string(&path)?)
    }

    fn delete_named_local(&self, sector_id: &str, name: &str) -> anyhow::Result<()> {
        let path = self.sessions_dir.join(format!("{}_{}.tos-session", sector_id, name));
        std::fs::remove_file(&path)?;
        Ok(())
    }

    fn list_local(&self, sector_id: &str) -> anyhow::Result<String> {
        let entries = match std::fs::read_dir(&self.sessions_dir) {
            Ok(e) => e,
            Err(_) => {
                let _ = std::fs::create_dir_all(&self.sessions_dir);
                return Ok("[]".to_string());
            }
        };
        let sessions: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if path.extension().unwrap_or_default() == "tos-session" {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem != "_live" {
                            if sector_id.is_empty() || sector_id == "global"
                                || stem.starts_with(&format!("{}_", sector_id))
                            {
                                let parts: Vec<&str> = stem.splitn(2, '_').collect();
                                if parts.len() == 2 {
                                    return Some(parts[1].to_string());
                                }
                            }
                        }
                    }
                }
                None
            })
            .collect();
        Ok(serde_json::to_string(&sessions).unwrap_or_else(|_| "[]".to_string()))
    }

    // ── Remote persistence (tos-sessiond daemon) ─────────────────────

    fn save_live_daemon(&self, state: &TosState, addr: &str) -> anyhow::Result<()> {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            let json = serde_json::to_string(state)?;
            let _ = stream.write_all(format!("session_live_write:{}\n", json).as_bytes());
            return Ok(());
        }
        Err(anyhow::anyhow!("connection failed to tos-sessiond at {}", addr))
    }

    fn save_named_daemon(&self, sector_id: &str, name: &str, state: &TosState, addr: &str) -> anyhow::Result<()> {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            let json = serde_json::to_string(state)?;
            let _ = stream.write_all(
                format!("session_save:{};{};{}\n", sector_id, name, json).as_bytes(),
            );
            return Ok(());
        }
        Err(anyhow::anyhow!("connection failed to tos-sessiond"))
    }

    fn load_named_daemon(&self, sector_id: &str, name: &str, addr: &str) -> anyhow::Result<String> {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            let _ = stream.write_all(
                format!("session_load:{};{}\n", sector_id, name).as_bytes(),
            );
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if reader.read_line(&mut response).is_ok() {
                let trimmed = response.trim();
                if trimmed.starts_with("ERROR") {
                    return Err(anyhow::anyhow!("{}", trimmed));
                }
                return Ok(trimmed.to_string());
            }
        }
        Err(anyhow::anyhow!("connection failed to tos-sessiond"))
    }

    fn delete_named_daemon(&self, sector_id: &str, name: &str, addr: &str) -> anyhow::Result<()> {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            let _ = stream.write_all(
                format!("session_delete:{};{}\n", sector_id, name).as_bytes(),
            );
            return Ok(());
        }
        Err(anyhow::anyhow!("connection failed to tos-sessiond"))
    }

    fn list_daemon(&self, addr: &str) -> anyhow::Result<String> {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            let _ = stream.write_all(b"session_list:\n");
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if reader.read_line(&mut response).is_ok() {
                return Ok(response.trim().to_string());
            }
        }
        Err(anyhow::anyhow!("connection failed to tos-sessiond"))
    }

    // ── Public API (routes to daemon or local) ───────────────────────

    /// Save the live session state. Routes to daemon if available, falls
    /// back to direct disk write when local_persistence is enabled.
    pub fn save_live(&self, state: &TosState) -> anyhow::Result<()> {
        // If local persistence is disabled, daemon is mandatory.
        if !self.local_persistence {
            if let Some(addr) = self.get_daemon_address() {
                return self.save_live_daemon(state, &addr);
            }
            return Err(anyhow::anyhow!(
                "tos-sessiond not found and local.persistence is disabled"
            ));
        }

        // Local-first: try daemon, fall back to disk.
        if let Some(addr) = self.get_daemon_address() {
            if self.save_live_daemon(state, &addr).is_ok() {
                return Ok(());
            }
        }
        self.save_live_local(state)
    }

    /// Debounced live write. Repeated calls within the debounce window
    /// collapse into a single write.
    pub fn debounced_save_live(&self, state: Arc<Mutex<TosState>>) {
        let now = std::time::Instant::now();
        *self.last_trigger.lock().unwrap() = now;

        let last_trigger_ref = self.last_trigger.clone();
        let service_ref = self.clone();
        let debounce = self.debounce_ms;

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(debounce)).await;
            let current_last = *last_trigger_ref.lock().unwrap();
            if current_last == now {
                let state_lock = state.lock().unwrap();
                if let Err(e) = service_ref.save_live(&state_lock) {
                    tracing::warn!("Failed debounced live save: {}", e);
                }
            }
        });
    }

    /// Save a named session.
    pub fn save(&self, sector_id: &str, name: &str, state: &TosState) -> anyhow::Result<()> {
        if let Some(addr) = self.get_daemon_address() {
            if self.save_named_daemon(sector_id, name, state, &addr).is_ok() {
                return Ok(());
            }
        }
        if self.local_persistence {
            return self.save_named_local(sector_id, name, state);
        }
        Err(anyhow::anyhow!("no persistence path available"))
    }

    /// Load a named session.
    pub fn load(&self, sector_id: &str, name: &str) -> anyhow::Result<String> {
        if let Some(addr) = self.get_daemon_address() {
            if let Ok(data) = self.load_named_daemon(sector_id, name, &addr) {
                return Ok(data);
            }
        }
        if self.local_persistence {
            return self.load_named_local(sector_id, name);
        }
        Err(anyhow::anyhow!("no persistence path available"))
    }

    /// Delete a named session.
    pub fn delete(&self, sector_id: &str, name: &str) -> anyhow::Result<()> {
        if let Some(addr) = self.get_daemon_address() {
            if self.delete_named_daemon(sector_id, name, &addr).is_ok() {
                return Ok(());
            }
        }
        if self.local_persistence {
            return self.delete_named_local(sector_id, name);
        }
        Err(anyhow::anyhow!("no persistence path available"))
    }

    /// List named sessions for a sector.
    pub fn list(&self, sector_id: &str) -> anyhow::Result<String> {
        if let Some(addr) = self.get_daemon_address() {
            if let Ok(data) = self.list_daemon(&addr) {
                return Ok(data);
            }
        }
        if self.local_persistence {
            return self.list_local(sector_id);
        }
        Err(anyhow::anyhow!("no persistence path available"))
    }
}
