//! Tests for Remote Sectors (Phase 12)

use tos_core::*;
use tos_core::system::remote::{RemoteNodeInfo, RemoteStatus, SyncPacket};
use uuid::Uuid;

#[test]
fn test_remote_manager_registration() {
    let mut state = TosState::new();
    let node_id = Uuid::new_v4();
    
    let info = RemoteNodeInfo {
        id: node_id,
        hostname: "test-node".to_string(),
        address: "192.168.1.100".to_string(),
        os_type: "TOS".to_string(),
        version: "1.0.0".to_string(),
        status: RemoteStatus::Online,
    };
    
    state.remote_manager.register_node(info.clone());
    assert!(state.remote_manager.nodes.contains_key(&node_id));
    assert_eq!(state.remote_manager.nodes.get(&node_id).unwrap().hostname, "test-node");
}

#[test]
fn test_remote_connection_lifecycle() {
    let mut state = TosState::new();
    let node_id = Uuid::new_v4();
    
    let info = RemoteNodeInfo {
        id: node_id,
        hostname: "remote-target".to_string(),
        address: "10.0.0.5".to_string(),
        os_type: "TOS".to_string(),
        version: "1.1.0".to_string(),
        status: RemoteStatus::Online,
    };
    
    state.remote_manager.register_node(info);
    
    // Test connect
    let result = state.remote_manager.connect(node_id, ConnectionType::TOSNative);
    assert!(result.is_ok());
    assert!(state.remote_manager.active_connections.contains_key(&node_id));
    
    // Test sector creation
    let sector = state.remote_manager.create_remote_sector(node_id);
    assert!(sector.is_some());
    let sector = sector.unwrap();
    assert_eq!(sector.id, node_id);
    assert!(sector.name.contains("remote-target"));
    assert_eq!(sector.connection_type, ConnectionType::TOSNative);
    
    // Test sync
    assert!(state.remote_manager.sync_node(node_id).is_ok());
    
    // Test disconnect
    state.remote_manager.disconnect(node_id);
    assert!(!state.remote_manager.active_connections.contains_key(&node_id));
}

#[test]
fn test_tos_native_sync_packet() {
    let mut state = TosState::new();
    let remote_node_id = Uuid::new_v4();
    
    // Create a mock sector that would come from a remote
    let remote_sector = Sector {
        id: remote_node_id,
        name: "Remote Alpha".to_string(),
        color: "#ffffff".to_string(),
        hubs: vec![],
        active_hub_index: 0,
        host: "remote-host".to_string(),
        connection_type: ConnectionType::TOSNative,
        participants: vec![],
        portal_active: false,
        portal_url: None,
    };
    
    let packet = SyncPacket::SectorState(remote_sector.clone());
    
    // Process the packet
    let result = state.remote_manager.process_packet(remote_node_id, packet, &mut state.sectors);
    assert!(result.is_ok());
    
    // Check if the sector was added/updated
    assert!(state.sectors.iter().any(|s| s.id == remote_node_id && s.name == "Remote Alpha"));
}

#[test]
fn test_terminal_delta_sync() {
    let mut state = TosState::new();
    let hub_id = state.sectors[0].hubs[0].id;
    
    let packet = SyncPacket::TerminalDelta {
        hub_id,
        line: "Remote Command Output".to_string(),
    };
    
    let _ = state.remote_manager.process_packet(Uuid::new_v4(), packet, &mut state.sectors);
    
    assert!(state.sectors[0].hubs[0].terminal_output.contains(&"Remote Command Output".to_string()));
}

#[test]
fn test_remote_sync_failure_when_not_connected() {
    let mut state = TosState::new();
    let node_id = Uuid::new_v4();
    
    let result = state.remote_manager.sync_node(node_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not connected");
}
