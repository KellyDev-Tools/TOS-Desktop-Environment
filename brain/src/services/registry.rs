//! Service Registry — runtime tracking of the daemon constellation.

pub use tos_common::services::{ServiceEntry, ServiceRegistry};

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
