//! Service Registry — runtime tracking of the daemon constellation.
//!
//! The Brain maintains a registry of all active satellite daemons
//! (settingsd, loggerd, marketplaced, priorityd, sessiond). Each daemon
//! registers on startup by connecting to the Brain's registration port
//! and announcing its name and ephemeral port. The registry enables:
//!
//! - `tos ports` CLI command (via IPC) to list all active services
//! - Health-check polling to detect daemon crashes
//! - Dynamic service discovery without hardcoded port numbers

use std::collections::HashMap;
use std::time::Instant;

/// A single registered daemon.
#[derive(Debug, Clone)]
pub struct ServiceEntry {
    /// Daemon identifier (e.g. "tos-settingsd").
    pub name: String,
    /// TCP port the daemon is listening on.
    pub port: u16,
    /// Hostname or IP address (always "127.0.0.1" for local daemons).
    pub host: String,
    /// Last time the daemon reported healthy.
    pub last_heartbeat: Instant,
    /// Whether the daemon is currently considered alive.
    pub alive: bool,
}

/// Tracks all Brain-managed daemons and their ports.
#[derive(Debug)]
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
    ///
    /// If a daemon with the same name is already registered, its entry
    /// is updated (supporting daemon restarts).
    pub fn register(&mut self, name: &str, port: u16, host: &str) {
        self.services.insert(name.to_string(), ServiceEntry {
            name: name.to_string(),
            port,
            host: host.to_string(),
            last_heartbeat: Instant::now(),
            alive: true,
        });
    }

    /// Remove a daemon from the registry (clean shutdown).
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

    /// Mark a daemon as dead (failed health check).
    pub fn mark_dead(&mut self, name: &str) {
        if let Some(entry) = self.services.get_mut(name) {
            entry.alive = false;
        }
    }

    /// Record a heartbeat from a daemon.
    pub fn heartbeat(&mut self, name: &str) {
        if let Some(entry) = self.services.get_mut(name) {
            entry.last_heartbeat = Instant::now();
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

    /// Return a human-readable port table (for `tos ports` IPC command).
    pub fn port_table(&self) -> String {
        let mut lines = vec![
            format!("{:<22} {:<8} {:<16} {}", "SERVICE", "PORT", "HOST", "STATUS"),
            format!("{}", "-".repeat(56)),
            format!("{:<22} {:<8} {:<16} {}", "tos-brain (anchor)", self.anchor_port, "0.0.0.0", "ACTIVE"),
        ];

        let mut entries: Vec<&ServiceEntry> = self.services.values().collect();
        entries.sort_by_key(|e| &e.name);

        for entry in entries {
            let status = if entry.alive { "ACTIVE" } else { "DEAD" };
            lines.push(format!("{:<22} {:<8} {:<16} {}", entry.name, entry.port, entry.host, status));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut reg = ServiceRegistry::new(7000);
        reg.register("tos-settingsd", 7002, "127.0.0.1");
        reg.register("tos-loggerd", 54321, "127.0.0.1");

        assert_eq!(reg.port_of("tos-settingsd"), Some(7002));
        assert_eq!(reg.port_of("tos-loggerd"), Some(54321));
        assert_eq!(reg.port_of("nonexistent"), None);
        assert_eq!(reg.anchor_port(), 7000);
    }

    #[test]
    fn deregister_removes_service() {
        let mut reg = ServiceRegistry::new(7000);
        reg.register("tos-sessiond", 12345, "127.0.0.1");
        assert!(reg.get("tos-sessiond").is_some());
        reg.deregister("tos-sessiond");
        assert!(reg.get("tos-sessiond").is_none());
    }

    #[test]
    fn re_register_updates_port() {
        let mut reg = ServiceRegistry::new(7000);
        reg.register("tos-marketplaced", 11111, "127.0.0.1");
        reg.register("tos-marketplaced", 22222, "127.0.0.1");
        assert_eq!(reg.port_of("tos-marketplaced"), Some(22222));
    }

    #[test]
    fn mark_dead_and_heartbeat() {
        let mut reg = ServiceRegistry::new(7000);
        reg.register("tos-priorityd", 55555, "127.0.0.1");
        assert!(reg.get("tos-priorityd").unwrap().alive);

        reg.mark_dead("tos-priorityd");
        assert!(!reg.get("tos-priorityd").unwrap().alive);

        reg.heartbeat("tos-priorityd");
        assert!(reg.get("tos-priorityd").unwrap().alive);
    }

    #[test]
    fn list_filters_alive_only() {
        let mut reg = ServiceRegistry::new(7000);
        reg.register("a-alive", 1000, "127.0.0.1");
        reg.register("b-dead", 2000, "127.0.0.1");
        reg.mark_dead("b-dead");

        assert_eq!(reg.list_all().len(), 2);
        assert_eq!(reg.list_alive().len(), 1);
        assert_eq!(reg.list_alive()[0].name, "a-alive");
    }

    #[test]
    fn port_table_includes_anchor() {
        let mut reg = ServiceRegistry::new(7000);
        reg.register("tos-settingsd", 7002, "127.0.0.1");
        let table = reg.port_table();
        assert!(table.contains("tos-brain (anchor)"));
        assert!(table.contains("7000"));
        assert!(table.contains("tos-settingsd"));
        assert!(table.contains("7002"));
    }
}
