use std::sync::Arc;
use tos_common::state::*;
use tos_common::ipc::*;
use uuid::Uuid;
use tos_common::brain::shell::ShellApi;
use tos_common::brain::module_manager::ModuleManager;
use tos_common::services::ServiceManager;
use tos_common::brain::ipc_handler::IpcHandler;

#[tokio::test]
async fn test_collaboration_presence_and_cursor_sync() {
    // Tests Phase 5.9 - Collaboration sync tracking over WebRTC presence events
    
    let state = Arc::new(std::sync::Mutex::new(TosState::default()));
    let config = tos_common::config::TosConfig::default();
    let services = Arc::new(ServiceManager::with_config(&config));
    
    let modules = Arc::new(ModuleManager::new(std::path::PathBuf::from("./dev/fixtures")));
    let sid = state.lock().unwrap().sectors[0].id;
    let hid = state.lock().unwrap().sectors[0].hubs[0].id;
    let shell_api = Arc::new(std::sync::Mutex::new(ShellApi::new(state.clone(), modules, services.ai.clone(), services.heuristic.clone(), sid, hid).unwrap()));
    
    let handler = IpcHandler::new(state.clone(), shell_api, services);
    
    let guest_id = Uuid::new_v4();
    
    // 1. Simulate Remote Guest Join (WebRTC Presence)
    let presence_json = serde_json::json!({
        "type": "presence",
        "user": guest_id,
        "status": "active",
        "level": 2, // level is u8
        "active_viewport_title": "Cargo.toml - Viewer Mode",
        "left_chip_state": "Collapsed",
        "right_chip_state": "Expanded"
    });
    
    let req = format!("webrtc_presence:{}", presence_json.to_string());
    assert_eq!(handler.dispatch(&req), "OK");
    
    // Validate guest was ingested properly into the Active Sector
    {
        let st = state.lock().unwrap();
        let sector = &st.sectors[0];
        assert_eq!(sector.participants.len(), 1);
        let guest = &sector.participants[0];
        assert_eq!(guest.id, guest_id);
        assert_eq!(guest.viewport_title.as_deref(), Some("Cargo.toml - Viewer Mode"));
    }
    
    // 2. Simulate Cursor Sync Event
    let cursor_json = serde_json::json!({
        "type": "cursor_sync",
        "user": guest_id,
        "x": 240.5,
        "y": 890.1,
        "target": "main.rs"
    });
    
    let c_req = format!("webrtc_presence:{}", cursor_json.to_string());
    assert_eq!(handler.dispatch(&c_req), "OK");
    
    // Validate Cursor Offsets Applied
    {
        let st = state.lock().unwrap();
        let guest = &st.sectors[0].participants[0];
        assert_eq!(guest.cursor_x, Some(240.5));
        assert_eq!(guest.cursor_y, Some(890.1));
        assert_eq!(guest.cursor_target.as_deref(), Some("main.rs"));
    }
}
