//! Service Registry — runtime tracking of the daemon constellation.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A single registered daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    /// Daemon identifier (e.g. "tos-settingsd").
    pub name: String,
    /// TCP port the daemon is listening on.
    pub port: u16,
    /// Hostname or IP address (always "127.0.0.1" for local daemons).
    pub host: String,
    /// Last time the daemon reported healthy (nanoseconds since epoch).
    pub last_heartbeat_ns: u64,
    /// Whether the daemon is currently considered alive.
    pub alive: bool,
}

/// Tracks all Brain-managed daemons and their ports.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceEntry>,
    /// The Brain's own anchor port — the well-known entry point.
    anchor_port: u16,
}

impl ServiceRegistry {
    /// Creates a new registry with the given anchor port.
    pub fn new(anchor_port: u16) -> Self {
        Self {
            services: HashMap::new(),
            anchor_port,
        }
    }

    /// Returns the Brain anchor port.
    pub fn anchor_port(&self) -> u16 {
        self.anchor_port
    }

    /// Register a daemon with the given name and port.
    pub fn register(&mut self, name: &str, port: u16, host: &str) {
        let now = now_ns();
        self.services.insert(
            name.to_string(),
            ServiceEntry {
                name: name.to_string(),
                port,
                host: host.to_string(),
                last_heartbeat_ns: now,
                alive: true,
            },
        );
    }

    /// Remove a daemon from the registry.
    pub fn deregister(&mut self, name: &str) {
        self.services.remove(name);
    }

    /// Look up a service by name.
    pub fn get(&self, name: &str) -> Option<&ServiceEntry> {
        self.services.get(name)
    }

    /// Look up a service's port by name.
    pub fn port_of(&self, name: &str) -> Option<u16> {
        self.services.get(name).map(|e| e.port)
    }

    /// Mark a daemon as dead.
    pub fn mark_dead(&mut self, name: &str) {
        if let Some(entry) = self.services.get_mut(name) {
            entry.alive = false;
        }
    }

    /// Record a heartbeat from a daemon.
    pub fn heartbeat(&mut self, name: &str) {
        if let Some(entry) = self.services.get_mut(name) {
            entry.last_heartbeat_ns = now_ns();
            entry.alive = true;
        }
    }

    /// List all currently registered services.
    pub fn list_all(&self) -> Vec<&ServiceEntry> {
        self.services.values().collect()
    }

    /// List only alive services.
    pub fn list_alive(&self) -> Vec<&ServiceEntry> {
        self.services.values().filter(|e| e.alive).collect()
    }

    /// Return a human-readable port table.
    pub fn port_table(&self) -> String {
        let mut lines = vec![
            format!(
                "{:<22} {:<8} {:<16} {}",
                "SERVICE", "PORT", "HOST", "STATUS"
            ),
            format!("{}", "-".repeat(56)),
            format!(
                "{:<22} {:<8} {:<16} {}",
                "tos-brain (anchor)", self.anchor_port, "0.0.0.0", "ACTIVE"
            ),
        ];

        let mut entries: Vec<&ServiceEntry> = self.services.values().collect();
        entries.sort_by_key(|e| &e.name);

        for entry in entries {
            let status = if entry.alive { "ACTIVE" } else { "DEAD" };
            lines.push(format!(
                "{:<22} {:<8} {:<16} {}",
                entry.name, entry.port, entry.host, status
            ));
        }

        lines.join("\n")
    }
}

fn now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
