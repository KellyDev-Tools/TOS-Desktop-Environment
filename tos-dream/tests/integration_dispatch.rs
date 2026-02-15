use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tos_core::{TosState, HierarchyLevel};
use tos_core::system::ipc::IpcDispatcher;

#[test]
fn test_ipc_navigation_integration() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    // 1. Test Sector Selection via IPC
    dispatcher.handle_request("select_sector:1");
    {
        let s = state.lock().unwrap();
        assert_eq!(s.viewports[s.active_viewport_index].sector_index, 1);
        assert_eq!(s.current_level, HierarchyLevel::CommandHub);
    }

    // 2. Test Zoom In via Semantic Event
    dispatcher.handle_request("semantic_event:ZoomIn");
    {
        let s = state.lock().unwrap();
        assert_eq!(s.current_level, HierarchyLevel::ApplicationFocus);
        
        // Verify rendering integration (Modular AppRenderer)
        let html = s.render_current_view();
        assert!(html.contains("application-container"));
        assert!(html.contains("tactical-bezel"));
    }

    // 3. Test Zoom Out via Direct Command
    dispatcher.handle_request("zoom_out");
    {
        let s = state.lock().unwrap();
        assert_eq!(s.current_level, HierarchyLevel::CommandHub);
        
        // Verify rendering integration (Modular HubRenderer)
        let html = s.render_current_view();
        assert!(html.contains("command-hub"));
        assert!(html.contains("hub-header"));
    }
}

#[test]
fn test_ipc_viewport_splitting_integration() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    // Initial state: 1 viewport
    assert_eq!(state.lock().unwrap().viewports.len(), 1);

    // Trigger Split via IPC
    dispatcher.handle_request("split_viewport");
    
    let s = state.lock().unwrap();
    assert_eq!(s.viewports.len(), 2);
    assert_eq!(s.current_level, HierarchyLevel::SplitView);
    
    // Check rendering of split view
    let html = s.render_current_view();
    assert!(html.contains("split-viewport-grid"));
    assert!(html.contains("viewport-cell"));
}

#[test]
fn test_ipc_remote_sector_integration() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    let initial_sector_count = state.lock().unwrap().sectors.len();

    // Add Remote Sector
    dispatcher.handle_request("add_remote_sector");
    
    let s = state.lock().unwrap();
    assert_eq!(s.sectors.len(), initial_sector_count + 1);
    assert!(matches!(s.sectors.last().unwrap().connection_type, tos_core::ConnectionType::SSH));
    
    // Verify renderer shows remote status
    let html = s.render_current_view(); // Global Overview
    assert!(html.contains("REMOTE"));
    assert!(html.contains("10.0.4.15"));
}

#[test]
fn test_ipc_dangerous_command_block_integration() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    // Nav to hub
    dispatcher.handle_request("select_sector:0");
    
    // Submit dangerous command
    dispatcher.handle_request("prompt_submit:rm -rf /");
    
    let s = state.lock().unwrap();
    let sector = &s.sectors[0];
    let hub = &sector.hubs[0];
    
    // Should be blocked and pending confirmation
    assert_eq!(hub.confirmation_required, Some("rm -rf /".to_string()));
    
    // Verify renderer shows the dangerous overlay
    let html = s.render_current_view();
    assert!(html.contains("DANGEROUS COMMAND DETECTED"));
    assert!(html.contains("EXECUTION BLOCKED"));
}

#[test]
fn test_module_rendering_integration() {
    let mut state_raw = tos_core::TosState::new();
    
    // Add a mockup module
    #[derive(Debug)]
    struct MockModule;
    impl tos_core::TosModule for MockModule {
        fn name(&self) -> String { "Mock".to_string() }
        fn version(&self) -> String { "1.0.0".to_string() }
        fn render_override(&self, level: tos_core::HierarchyLevel) -> Option<String> {
            if level == tos_core::HierarchyLevel::ApplicationFocus {
                Some("<div class='mock-module'>MOCK DATA</div>".to_string())
            } else {
                None
            }
        }
    }
    
    state_raw.modules.push(Box::new(MockModule));
    let state = Arc::new(Mutex::new(state_raw));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    // Zoom into app focus
    dispatcher.handle_request("select_sector:0"); // Hub
    dispatcher.handle_request("semantic_event:ZoomIn"); // App
    
    let s = state.lock().unwrap();
    assert_eq!(s.current_level, HierarchyLevel::ApplicationFocus);
    
    // Verify renderer includes module content
    let html = s.render_current_view();
    assert!(html.contains("MOCK DATA"));
    assert!(html.contains("mock-module"));
}

#[test]
fn test_complex_multi_viewport_state_integration() {
    let state = Arc::new(Mutex::new(tos_core::TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    // 1. Setup two viewports in split view
    dispatcher.handle_request("split_viewport");
    
    // 2. Set Viewport 0 to Hub level
    {
        let mut s = state.lock().unwrap();
        s.active_viewport_index = 0;
        s.current_level = HierarchyLevel::CommandHub;
        s.viewports[0].current_level = HierarchyLevel::CommandHub;
    }

    // 3. Set Viewport 1 to Application level
    {
        let mut s = state.lock().unwrap();
        s.active_viewport_index = 1;

        let (s_idx, h_idx) = {
            let v = &s.viewports[1];
            (v.sector_index, v.hub_index)
        };

        // Inject an application so zoom_in works
        s.sectors[s_idx].hubs[h_idx].applications.push(
            tos_core::Application {
                id: uuid::Uuid::new_v4(),
                title: "Tactical Map".to_string(),
                app_class: "tos.map".to_string(),
                is_minimized: false,
            }
        );
        s.zoom_in(); // Hub -> App
        s.current_level = HierarchyLevel::SplitView; // Maintain split view for rendering
    }

    let s = state.lock().unwrap();
    let html = s.render_current_view();

    // Verify both viewports are rendered with correct modular logic
    assert!(html.contains("command-hub")); // Viewport 0
    assert!(html.contains("application-container")); // Viewport 1
    
    // Verify depth-based render modes are applied correctly in split view
    // Viewport 1 is focused (App), Viewport 0 is Background (Hub)
    assert!(html.contains("render-Throttled")); 
    assert!(html.contains("render-Full"));
}

#[test]
fn test_pty_to_ui_integration() {
    let state = Arc::new(Mutex::new(tos_core::TosState::new()));
    let _ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    
    // Simulate PTY output and set viewport to Hub level
    {
        let mut s = state.lock().unwrap();
        let (s_idx, h_idx) = {
            let v = &s.viewports[0];
            (v.sector_index, v.hub_index)
        };
        s.viewports[0].current_level = HierarchyLevel::CommandHub;
        
        let hub = &mut s.sectors[s_idx].hubs[h_idx];
        hub.terminal_output.push("TACTICAL DATA RECEIVED".to_string());
    }

    // Verify HubRenderer displays the PTY output
    let s = state.lock().unwrap();
    let html = s.render_viewport(&s.viewports[0]);
    assert!(html.contains("TACTICAL DATA RECEIVED"));
}

#[test]
fn test_performance_alert_integration() {
    let state = Arc::new(Mutex::new(tos_core::TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    // Trigger performance alert and render view
    let html = {
        let mut s = state.lock().unwrap();
        s.performance_alert = true;
        s.fps = 15.2;
        s.render_current_view()
    };

    // Verify tactical alert overlay is present
    assert!(html.contains("PERFORMANCE CRITICAL"));
    assert!(html.contains("15.2"));

    // Test IPC optimization action (lock released above, so dispatcher can acquire it)
    dispatcher.handle_request("optimize_system");
    
    // Verify optimization was applied
    let s = state.lock().unwrap();
    assert_eq!(s.performance_alert, false);
    assert_eq!(s.fps, 60.0);
}

#[test]
fn test_bezel_interaction_integration() {
    let state = Arc::new(Mutex::new(tos_core::TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    // Nav to app
    dispatcher.handle_request("select_sector:0");
    dispatcher.handle_request("semantic_event:ZoomIn");

    // Initially collapsed
    {
        let s = state.lock().unwrap();
        let html = s.render_current_view();
        assert!(html.contains("tactical-bezel collapsed"));
    }

    // Toggle Bezel via IPC
    dispatcher.handle_request("toggle_bezel");

    // Now expanded
    {
        let s = state.lock().unwrap();
        let html = s.render_current_view();
        assert!(html.contains("tactical-bezel expanded"));
        assert!(html.contains("ZOOM OUT")); // Content shown in expanded state
    }
}

#[test]
fn test_voice_command_integration() {
    let state = Arc::new(Mutex::new(tos_core::TosState::new()));
    let _ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&_ptys));

    // Initially prompt is empty
    {
        let s = state.lock().unwrap();
        assert_eq!(s.sectors[0].hubs[0].prompt, "");
    }

    // Trigger Voice Start
    dispatcher.handle_request("semantic_event:VoiceCommandStart");
    
    // Should have staged listening message
    {
        let s = state.lock().unwrap();
        assert!(s.sectors[0].hubs[0].prompt.contains("LISTENING"));
    }
}

#[test]
fn test_tactical_reset_integration() {
    let state = Arc::new(Mutex::new(tos_core::TosState::new()));
    let _ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, tos_core::system::pty::PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&_ptys));

    // Nav to deep inspector
    dispatcher.handle_request("select_sector:0");
    dispatcher.handle_request("semantic_event:ZoomIn"); // App
    dispatcher.handle_request("semantic_event:ZoomIn"); // Inspector
    
    {
        let s = state.lock().unwrap();
        assert_eq!(s.current_level, HierarchyLevel::DetailInspector);
    }

    // Trigger Tactical Reset
    dispatcher.handle_request("tactical_reset");
    
    // Should be back at Global
    {
        let s = state.lock().unwrap();
        assert_eq!(s.current_level, HierarchyLevel::GlobalOverview);
    }
}
