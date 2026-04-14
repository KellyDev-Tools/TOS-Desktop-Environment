//! Tests for the tos-protocol crate.
//!
//! These validate that the shared Face-Brain contract types serialize,
//! deserialize, and cascade correctly. The Brain and Face both depend on
//! these guarantees.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use tos_common::collaboration::*;
    use tos_common::ipc::IpcDispatcher;
    use tos_common::modules::*;
    use tos_common::state::*;

    // -----------------------------------------------------------------------
    // State Defaults
    // -----------------------------------------------------------------------

    #[test]
    fn default_state_has_one_sector() {
        let state = TosState::default();
        assert_eq!(
            state.sectors.len(),
            1,
            "Default state must have exactly one sector"
        );
        assert_eq!(state.sectors[0].name, "Primary");
    }

    #[test]
    fn default_state_starts_at_global_overview() {
        let state = TosState::default();
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    }

    #[test]
    fn default_sector_has_one_hub_in_command_mode() {
        let state = TosState::default();
        let sector = &state.sectors[0];
        assert_eq!(sector.hubs.len(), 1);
        assert_eq!(sector.hubs[0].mode, CommandHubMode::Command);
    }

    #[test]
    fn default_state_has_available_modules() {
        let state = TosState::default();
        assert!(
            state.available_modules.len() >= 2,
            "Should ship with at least rectangular + cinematic modules"
        );
        assert!(
            state.available_themes.len() >= 3,
            "Should ship with at least 3 themes"
        );
        assert!(
            !state.available_ai_modules.is_empty(),
            "Should have at least one AI module"
        );
    }

    // -----------------------------------------------------------------------
    // Settings Cascading Resolution
    // -----------------------------------------------------------------------

    #[test]
    fn settings_resolve_global() {
        let mut store = SettingsStore::default();
        store
            .global
            .insert("theme".to_string(), "amber".to_string());

        assert_eq!(
            store.resolve("theme", None, None),
            Some("amber".to_string())
        );
    }

    #[test]
    fn settings_resolve_sector_overrides_global() {
        let mut store = SettingsStore::default();
        store
            .global
            .insert("theme".to_string(), "amber".to_string());

        let mut sector_settings = HashMap::new();
        sector_settings.insert("theme".to_string(), "red-alert".to_string());
        store
            .sectors
            .insert("sector_1".to_string(), sector_settings);

        // With sector context, sector value wins.
        assert_eq!(
            store.resolve("theme", Some("sector_1"), None),
            Some("red-alert".to_string())
        );
        // Without sector context, global value returns.
        assert_eq!(
            store.resolve("theme", None, None),
            Some("amber".to_string())
        );
    }

    #[test]
    fn settings_resolve_app_overrides_sector_and_global() {
        let mut store = SettingsStore::default();
        store
            .global
            .insert("font_size".to_string(), "12".to_string());

        let mut sector_settings = HashMap::new();
        sector_settings.insert("font_size".to_string(), "14".to_string());
        store.sectors.insert("s1".to_string(), sector_settings);

        let mut app_settings = HashMap::new();
        app_settings.insert("font_size".to_string(), "16".to_string());
        store
            .applications
            .insert("editor".to_string(), app_settings);

        assert_eq!(
            store.resolve("font_size", Some("s1"), Some("editor")),
            Some("16".to_string())
        );
    }

    #[test]
    fn settings_resolve_missing_returns_none() {
        let store = SettingsStore::default();
        assert_eq!(store.resolve("nonexistent", None, None), None);
    }

    // -----------------------------------------------------------------------
    // Serialization Round-Trip (Face-Brain Contract Stability)
    // -----------------------------------------------------------------------

    #[test]
    fn tos_state_serialization_roundtrip() {
        let state = TosState::default();
        let json = serde_json::to_string(&state).expect("TosState must serialize");
        let deserialized: TosState =
            serde_json::from_str(&json).expect("TosState must deserialize");

        assert_eq!(deserialized.sectors.len(), state.sectors.len());
        assert_eq!(deserialized.current_level, state.current_level);
        assert_eq!(
            deserialized.active_terminal_module,
            state.active_terminal_module
        );
    }

    #[test]
    fn collaboration_payload_serialization_roundtrip() {
        let payload = WebRtcPayload::Presence {
            user: uuid::Uuid::new_v4(),
            status: PresenceStatus::Active,
            level: 2,
            active_viewport_title: Some("dev".to_string()),
            left_chip_state: None,
            right_chip_state: None,
        };

        let json = serde_json::to_string(&payload).expect("WebRtcPayload must serialize");
        let deserialized: WebRtcPayload =
            serde_json::from_str(&json).expect("WebRtcPayload must deserialize");

        match deserialized {
            WebRtcPayload::Presence { status, level, .. } => {
                assert_eq!(status, PresenceStatus::Active);
                assert_eq!(level, 2);
            }
            _ => panic!("Expected Presence variant"),
        }
    }

    #[test]
    fn ai_query_response_serialization() {
        let query = AiQuery {
            prompt: "explain this error".to_string(),
            context: vec!["cargo build".to_string()],
            stream: false,
        };
        let json = serde_json::to_string(&query).expect("AiQuery must serialize");
        let _: AiQuery = serde_json::from_str(&json).expect("AiQuery must deserialize");

        let response = AiResponse {
            id: uuid::Uuid::new_v4(),
            choice: AiChoice {
                role: "assistant".to_string(),
                content: "This is a borrow checker error.".to_string(),
            },
            usage: AiUsage { tokens: 42 },
            status: AiStatus::Complete,
        };
        let json = serde_json::to_string(&response).expect("AiResponse must serialize");
        let _: AiResponse = serde_json::from_str(&json).expect("AiResponse must deserialize");
    }

    // -----------------------------------------------------------------------
    // IPC Dispatcher Trait Object Safety
    // -----------------------------------------------------------------------

    struct MockDispatcher;
    impl IpcDispatcher for MockDispatcher {
        fn dispatch(&self, request: &str) -> String {
            format!("ECHO:{}", request)
        }
    }

    #[test]
    fn ipc_dispatcher_is_object_safe() {
        let dispatcher: Box<dyn IpcDispatcher> = Box::new(MockDispatcher);
        let response = dispatcher.dispatch("get_state");
        assert_eq!(response, "ECHO:get_state");
    }

    // -----------------------------------------------------------------------
    // Hierarchy Level Values
    // -----------------------------------------------------------------------

    #[test]
    fn hierarchy_levels_have_correct_integer_values() {
        assert_eq!(HierarchyLevel::GlobalOverview as u8, 1);
        assert_eq!(HierarchyLevel::CommandHub as u8, 2);
        assert_eq!(HierarchyLevel::ApplicationFocus as u8, 3);
        assert_eq!(HierarchyLevel::DetailView as u8, 4);
        assert_eq!(HierarchyLevel::BufferView as u8, 5);
    }

    // -----------------------------------------------------------------------
    // Sector & Hub Construction
    // -----------------------------------------------------------------------

    #[test]
    fn sector_can_hold_multiple_hubs() {
        let hub_a = CommandHub {
            id: uuid::Uuid::new_v4(),
            mode: CommandHubMode::Command,
            prompt: String::new(),
            current_directory: std::path::PathBuf::from("/"),
            terminal_output: vec![],
            buffer_limit: 500,
            shell_listing: None,
            activity_listing: None,
            search_results: None,
            staged_command: None,
            ai_explanation: None,
            json_context: None,
            shell_module: None,
            focused_pane_id: None,
            split_layout: None,
            ai_history: vec![],
            active_thoughts: vec![],
            version: 0,
            last_exit_status: None,
            is_running: false,
        };
        let hub_b = CommandHub {
            id: uuid::Uuid::new_v4(),
            mode: CommandHubMode::Ai,
            prompt: "explain".to_string(),
            current_directory: std::path::PathBuf::from("/home"),
            terminal_output: vec![],
            buffer_limit: 500,
            shell_listing: None,
            activity_listing: None,
            search_results: None,
            staged_command: Some("git status".to_string()),
            ai_explanation: Some("Shows working tree status".to_string()),
            json_context: None,
            shell_module: Some("tos-shell-fish".to_string()),
            focused_pane_id: None,
            split_layout: None,
            ai_history: vec![],
            active_thoughts: vec![],
            version: 0,
            last_exit_status: Some(0),
            is_running: false,
        };

        let sector = Sector {
            id: uuid::Uuid::new_v4(),
            name: "multi-hub".to_string(),
            hubs: vec![hub_a, hub_b],
            active_hub_index: 1,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: TrustTier::Standard,
            priority: 3,
            active_apps: vec![],
            active_app_index: 0,
            participants: vec![],
            kanban_board: None,
            version: 0,
        };

        assert_eq!(sector.hubs.len(), 2);
        assert_eq!(
            sector.hubs[sector.active_hub_index].mode,
            CommandHubMode::Ai
        );
    }
}
