use tos_core::TosState;
use tos_core::ui::render::ViewRenderer;
use uuid::Uuid;

#[test]
fn test_portal_wiring_and_remote_metrics() {
    let mut state = TosState::new_fresh();
    
    // 1. Test Portal Activation
    let sector_id = state.sectors[0].id;
    state.active_viewport_index = 0;
    state.viewports[0].sector_index = 0;
    
    // Toggle portal (should request approval first since security bypass is off by default)
    state.toggle_portal();
    assert!(state.approval_requested_sector.is_some());
    assert_eq!(state.approval_requested_sector.unwrap(), sector_id);
    
    // Approve portal
    state.approve_portal();
    assert!(state.sectors[0].portal_active);
    assert!(state.sectors[0].portal_url.is_some());
    assert!(state.sectors[0].portal_url.as_ref().unwrap().starts_with("https://tos.grid/portal/"));

    // 2. Test Remote Metrics
    // Simulate a remote connection
    let remote_id = state.sectors[0].id; // For testing, treat first sector as remote
    state.remote_manager.register_node(tos_core::system::remote::RemoteNodeInfo {
        id: remote_id,
        hostname: "TEST-NODE".to_string(),
        address: "127.0.0.1".to_string(),
        os_type: "TOS".to_string(),
        version: "1.0".to_string(),
        status: tos_core::system::remote::RemoteStatus::Online,
    });
    
    let mut conn = tos_core::system::remote::RemoteConnection::new(remote_id, tos_core::ConnectionType::TOSNative);
    conn.latency_ms = 42;
    conn.stream_quality = 85;
    state.remote_manager.active_connections.insert(remote_id, conn);
    
    // Verify renderer picks up metrics
    let renderer = tos_core::ui::render::remote::RemoteDesktopRenderer;
    let html = renderer.render(&state, &state.viewports[0], tos_core::RenderMode::Full);
    
    assert!(html.contains("42MS"));
    assert!(html.contains("set_stream_quality")); // Check if slider is present
}
