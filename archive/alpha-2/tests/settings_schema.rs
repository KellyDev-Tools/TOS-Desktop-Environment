//! Tests for the Alpha-2.2 Settings Schema Extensions.
//!
//! These validate that all new setting namespaces required by 2.2 features
//! are present in the default settings, persist correctly through
//! serialization, and cascade properly.

#[cfg(test)]
mod tests {
    use tos_protocol::state::SettingsStore;
    use std::collections::HashMap;

    // -----------------------------------------------------------------------
    // Onboarding Namespace (Onboarding Specification §2)
    // -----------------------------------------------------------------------

    #[test]
    fn default_settings_include_onboarding_state() {
        let defaults = test_default_settings();
        assert_eq!(defaults.global.get("tos.onboarding.first_run_complete"), Some(&"false".to_string()));
        assert_eq!(defaults.global.get("tos.onboarding.wizard_complete"), Some(&"false".to_string()));
        assert_eq!(defaults.global.get("tos.onboarding.hint_suppressed"), Some(&"false".to_string()));
        assert_eq!(defaults.global.get("tos.onboarding.sessions_count"), Some(&"0".to_string()));
        assert_eq!(defaults.global.get("tos.onboarding.commands_run"), Some(&"0".to_string()));
    }

    // -----------------------------------------------------------------------
    // Trust Namespace (Trust & Confirmation Specification §2, §6)
    // -----------------------------------------------------------------------

    #[test]
    fn default_settings_include_trust_config() {
        let defaults = test_default_settings();
        // Both command classes default to WARN (user must explicitly promote)
        assert_eq!(defaults.global.get("tos.trust.privilege_escalation"), Some(&"warn".to_string()));
        assert_eq!(defaults.global.get("tos.trust.recursive_bulk"), Some(&"warn".to_string()));
        assert_eq!(defaults.global.get("tos.trust.bulk_threshold"), Some(&"10".to_string()));
    }

    #[test]
    fn trust_sector_override_cascades_correctly() {
        let mut store = test_default_settings();

        // Global says WARN, sector overrides to TRUST
        let mut sector_overrides = HashMap::new();
        sector_overrides.insert("tos.trust.privilege_escalation".to_string(), "trust".to_string());
        store.sectors.insert("scratch".to_string(), sector_overrides);

        // Sector context returns the override
        assert_eq!(
            store.resolve("tos.trust.privilege_escalation", Some("scratch"), None),
            Some("trust".to_string())
        );
        // Without sector context, global returns
        assert_eq!(
            store.resolve("tos.trust.privilege_escalation", None, None),
            Some("warn".to_string())
        );
    }

    // -----------------------------------------------------------------------
    // AI Namespace (AI Co-Pilot Specification §9)
    // -----------------------------------------------------------------------

    #[test]
    fn default_settings_include_ai_config() {
        let defaults = test_default_settings();
        assert_eq!(defaults.global.get("tos.ai.default_backend"), Some(&"tos-ai-standard".to_string()));
        assert_eq!(defaults.global.get("tos.ai.chip_color"), Some(&"secondary".to_string()));
        assert_eq!(defaults.global.get("tos.ai.ghost_text_opacity"), Some(&"40".to_string()));
        assert_eq!(defaults.global.get("tos.ai.disabled"), Some(&"false".to_string()));
        assert_eq!(defaults.global.get("tos.ai.context_level"), Some(&"standard".to_string()));
    }

    // -----------------------------------------------------------------------
    // Bezel Namespace (Expanded Bezel Specification §7)
    // -----------------------------------------------------------------------

    #[test]
    fn default_settings_include_bezel_config() {
        let defaults = test_default_settings();
        assert_eq!(defaults.global.get("tos.interface.bezel.dismiss_behavior"), Some(&"stay_open".to_string()));
        assert_eq!(defaults.global.get("tos.interface.bezel.auto_collapse_timeout"), Some(&"5".to_string()));
    }

    // -----------------------------------------------------------------------
    // Split Viewport Namespace (Split Viewport Specification §6)
    // -----------------------------------------------------------------------

    #[test]
    fn default_settings_include_split_config() {
        let defaults = test_default_settings();
        assert_eq!(defaults.global.get("tos.interface.splits.divider_snap"), Some(&"true".to_string()));
    }

    // -----------------------------------------------------------------------
    // Network Namespace (Ecosystem Orchestration / Anchor Port)
    // -----------------------------------------------------------------------

    #[test]
    fn default_settings_include_network_config() {
        let defaults = test_default_settings();
        assert_eq!(defaults.global.get("tos.network.anchor_port"), Some(&"7000".to_string()));
        assert_eq!(defaults.global.get("tos.network.mdns_enabled"), Some(&"true".to_string()));
        assert_eq!(defaults.global.get("tos.network.remote_access"), Some(&"false".to_string()));
    }

    // -----------------------------------------------------------------------
    // Serialization Round-Trip
    // -----------------------------------------------------------------------

    #[test]
    fn extended_settings_survive_serialization() {
        let defaults = test_default_settings();
        let json = serde_json::to_string(&defaults).expect("Settings must serialize");
        let restored: SettingsStore = serde_json::from_str(&json).expect("Settings must deserialize");

        // Spot-check keys from each namespace
        assert_eq!(restored.global.get("tos.onboarding.first_run_complete"), Some(&"false".to_string()));
        assert_eq!(restored.global.get("tos.trust.privilege_escalation"), Some(&"warn".to_string()));
        assert_eq!(restored.global.get("tos.ai.default_backend"), Some(&"tos-ai-standard".to_string()));
        assert_eq!(restored.global.get("tos.interface.bezel.dismiss_behavior"), Some(&"stay_open".to_string()));
        assert_eq!(restored.global.get("tos.network.anchor_port"), Some(&"7000".to_string()));
    }

    // -----------------------------------------------------------------------
    // Helper: Build a SettingsStore with all 2.2 defaults populated
    // -----------------------------------------------------------------------

    fn test_default_settings() -> SettingsStore {
        tos_lib::services::settings::SettingsService::new().default_settings_public()
    }
}
