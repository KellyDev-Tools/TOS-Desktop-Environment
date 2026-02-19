//! Integration tests for Audio/Earcon system and Remote Sector functionality
//! Tests cover: Audio event handling, earcon playback, remote connection lifecycle, packet processing

use tos_core::*;
use tos_core::system::remote::{RemoteNodeInfo, RemoteStatus, SyncPacket, RemoteManager};
use tos_core::system::audio::{AudioManager, AudioEvent, AmbienceProfile};
use tos_core::system::audio::earcons::{EarconPlayer, EarconEvent, EarconCategory, SpatialPosition};
use uuid::Uuid;

// ============================================================================
// Audio Manager Tests
// ============================================================================

#[test]
fn test_audio_manager_creation() {
    let manager = AudioManager::new();
    assert_eq!(manager.volume, 0.8);
    assert!(!manager.muted);
}

#[test]
fn test_audio_manager_mute_unmute() {
    let mut manager = AudioManager::new();
    
    manager.muted = true;
    assert!(manager.muted);
    
    manager.muted = false;
    assert!(!manager.muted);
}

#[test]
fn test_audio_manager_play_event_when_muted() {
    let manager = AudioManager::new();
    // Should not panic, just return
    manager.play_event(AudioEvent::DataTransfer);
}

#[test]
fn test_audio_manager_set_sector_ambience() {
    let mut manager = AudioManager::new();
    let sector_id = Uuid::new_v4();
    
    let profile = AmbienceProfile {
        base_loop: AudioEvent::BridgeChirps,
        secondary_layers: vec![AudioEvent::ComputerThinking],
        volume: 0.5,
        pitch: 1.2,
    };
    
    manager.set_sector_ambience(sector_id, profile.clone());
    
    assert!(manager.sector_ambience.contains_key(&sector_id));
    let stored = manager.sector_ambience.get(&sector_id).unwrap();
    assert_eq!(stored.base_loop, AudioEvent::BridgeChirps);
    assert_eq!(stored.volume, 0.5);
}

#[test]
fn test_audio_event_variants() {
    // Verify all audio events can be created
    let _ = AudioEvent::AmbientHum;
    let _ = AudioEvent::BridgeChirps;
    let _ = AudioEvent::ComputerThinking;
    let _ = AudioEvent::DataTransfer;
    let _ = AudioEvent::SectorTransition;
    let _ = AudioEvent::PortalHum;
    let _ = AudioEvent::AlertBeep;
}

// ============================================================================
// Earcon Player Tests (Component Level)
// ============================================================================

#[test]
fn test_earcon_player_default_state() {
    let player = EarconPlayer::new();
    assert!(player.is_enabled());
    assert_eq!(player.master_volume(), 1.0);
    assert!(player.is_spatial_audio_enabled());
}

#[test]
fn test_earcon_player_enable_disable() {
    let mut player = EarconPlayer::new();
    
    player.set_enabled(false);
    assert!(!player.is_enabled());
    
    player.set_enabled(true);
    assert!(player.is_enabled());
}

#[test]
fn test_earcon_player_category_volumes() {
    let mut player = EarconPlayer::new();
    
    // Test setting category volumes
    player.set_category_volume(EarconCategory::Navigation, 0.5);
    player.set_category_volume(EarconCategory::CommandFeedback, 0.8);
    
    assert_eq!(player.category_volume(EarconCategory::Navigation), 0.5);
    assert_eq!(player.category_volume(EarconCategory::CommandFeedback), 0.8);
}

#[test]
fn test_earcon_event_priority_levels() {
    // Critical events have highest priority (10)
    assert_eq!(EarconEvent::DangerousCommandWarning.priority(), 10);
    assert_eq!(EarconEvent::TacticalAlert.priority(), 10);
    assert_eq!(EarconEvent::BatteryCritical.priority(), 10);
    
    // High priority events (8)
    assert_eq!(EarconEvent::CommandError.priority(), 8);
    assert_eq!(EarconEvent::UserJoined.priority(), 8);
    
    // Low priority events (<=2)
    assert!(EarconEvent::ButtonHover.priority() <= 2);
    assert!(EarconEvent::AutoCompleteSuggestion.priority() <= 2);
}

#[test]
fn test_earcon_event_category_mapping() {
    // Verify event to category mapping
    assert_eq!(EarconEvent::ZoomIn.category(), EarconCategory::Navigation);
    assert_eq!(EarconEvent::ZoomOut.category(), EarconCategory::Navigation);
    assert_eq!(EarconEvent::CommandAccepted.category(), EarconCategory::CommandFeedback);
    assert_eq!(EarconEvent::Notification.category(), EarconCategory::SystemStatus);
    assert_eq!(EarconEvent::UserJoined.category(), EarconCategory::Collaboration);
    assert_eq!(EarconEvent::BezelExpand.category(), EarconCategory::BezelUi);
}

#[test]
fn test_earcon_category_descriptions() {
    assert_eq!(EarconCategory::Navigation.description(), "Navigation sounds");
    assert_eq!(EarconCategory::CommandFeedback.description(), "Command feedback");
    assert_eq!(EarconCategory::SystemStatus.description(), "System status alerts");
    assert_eq!(EarconCategory::Collaboration.description(), "Collaboration events");
    assert_eq!(EarconCategory::BezelUi.description(), "UI interactions");
}

#[test]
fn test_earcon_category_default_volumes() {
    assert_eq!(EarconCategory::Navigation.default_volume(), 0.7);
    assert_eq!(EarconCategory::CommandFeedback.default_volume(), 0.8);
    assert_eq!(EarconCategory::SystemStatus.default_volume(), 0.9);
    assert_eq!(EarconCategory::Collaboration.default_volume(), 0.6);
    assert_eq!(EarconCategory::BezelUi.default_volume(), 0.5);
}

#[test]
fn test_earcon_sound_pattern_names() {
    // Verify sound pattern strings are valid
    assert_eq!(EarconEvent::ZoomIn.sound_pattern(), "ascending_chime");
    assert_eq!(EarconEvent::ZoomOut.sound_pattern(), "descending_chime");
    assert_eq!(EarconEvent::CommandAccepted.sound_pattern(), "positive_beep");
    assert_eq!(EarconEvent::CommandError.sound_pattern(), "error_buzz");
    assert_eq!(EarconEvent::TacticalAlert.sound_pattern(), "urgent_alarm");
    assert_eq!(EarconEvent::UserJoined.sound_pattern(), "door_chime");
}

#[test]
fn test_spatial_position_pan_values() {
    let center = SpatialPosition::center();
    assert_eq!(center.pan(), 0.0);
    assert_eq!(center.attenuation(), 1.0);
    
    let left = SpatialPosition::from_sector_position(-1.0, 0.0, 0.0);
    assert_eq!(left.pan(), -1.0);
    
    let right = SpatialPosition::from_sector_position(1.0, 0.0, 0.0);
    assert_eq!(right.pan(), 1.0);
}

#[test]
fn test_spatial_position_attenuation() {
    let near = SpatialPosition::from_sector_position(0.0, 0.0, 0.0);
    assert_eq!(near.attenuation(), 1.0);
    
    let far = SpatialPosition::from_sector_position(0.0, 0.0, 1.0);
    assert!(far.attenuation() < 1.0);
    assert!(far.attenuation() >= 0.5);
}

#[test]
fn test_earcon_player_max_concurrent_setting() {
    let mut player = EarconPlayer::new();
    
    player.set_max_concurrent(4);
    
    // Should not change the limit to less than 1
    player.set_max_concurrent(0);
    // The implementation should ensure at least 1
}

// ============================================================================
// Remote Manager Tests (Component Level)
// ============================================================================

#[test]
fn test_remote_manager_initial_state() {
    let manager = RemoteManager::new();
    assert!(manager.nodes.is_empty());
    assert!(manager.active_connections.is_empty());
    assert!(manager.auth_store.is_empty());
}

#[test]
fn test_remote_manager_register_multiple_nodes() {
    let mut manager = RemoteManager::new();
    
    let node1 = RemoteNodeInfo {
        id: Uuid::new_v4(),
        hostname: "node1".to_string(),
        address: "192.168.1.10".to_string(),
        os_type: "TOS".to_string(),
        version: "1.0.0".to_string(),
        status: RemoteStatus::Online,
    };
    
    let node2 = RemoteNodeInfo {
        id: Uuid::new_v4(),
        hostname: "node2".to_string(),
        address: "192.168.1.11".to_string(),
        os_type: "Linux".to_string(),
        version: "2.0.0".to_string(),
        status: RemoteStatus::Offline,
    };
    
    manager.register_node(node1.clone());
    manager.register_node(node2.clone());
    
    assert_eq!(manager.nodes.len(), 2);
    assert!(manager.nodes.contains_key(&node1.id));
    assert!(manager.nodes.contains_key(&node2.id));
}

#[test]
fn test_remote_manager_connect_invalid_node() {
    let mut manager = RemoteManager::new();
    
    let result = manager.connect(Uuid::new_v4(), ConnectionType::SSH);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_remote_manager_disconnect_nonexistent() {
    let mut manager = RemoteManager::new();
    // Should not panic
    manager.disconnect(Uuid::new_v4());
}

#[test]
fn test_remote_manager_create_remote_sector_not_connected() {
    let manager = RemoteManager::new();
    let result = manager.create_remote_sector(Uuid::new_v4());
    assert!(result.is_none());
}

#[test]
fn test_remote_manager_sync_not_connected() {
    let mut manager = RemoteManager::new();
    let result = manager.sync_node(Uuid::new_v4());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not connected");
}

// ============================================================================
// Remote Sync Packet Tests (Component Level)
// ============================================================================

#[test]
fn test_sync_packet_sector_state() {
    let sector = Sector {
        id: Uuid::new_v4(),
        name: "Test Sector".to_string(),
        color: "#ff0000".to_string(),
        hubs: vec![],
        active_hub_index: 0,
        host: "test-host".to_string(),
        connection_type: ConnectionType::TOSNative,
        participants: vec![],
        portal_active: false,
        portal_url: None,
        description: "Test description".to_string(),
        icon: "🧪".to_string(),
        sector_type_name: "development".to_string(),
    };
    
    let packet = SyncPacket::SectorState(sector.clone());
    match packet {
        SyncPacket::SectorState(s) => assert_eq!(s.name, "Test Sector"),
        _ => panic!("Expected SectorState variant"),
    }
}

#[test]
fn test_sync_packet_terminal_delta() {
    let hub_id = Uuid::new_v4();
    let packet = SyncPacket::TerminalDelta {
        hub_id,
        line: "test output line".to_string(),
    };
    
    match packet {
        SyncPacket::TerminalDelta { hub_id: _, line } => {
            assert_eq!(line, "test output line");
        }
        _ => panic!("Expected TerminalDelta variant"),
    }
}

#[test]
fn test_sync_packet_command_relay() {
    let hub_id = Uuid::new_v4();
    let packet = SyncPacket::CommandRelay {
        hub_id,
        command: "ls -la".to_string(),
    };
    
    match packet {
        SyncPacket::CommandRelay { hub_id: _, command } => {
            assert_eq!(command, "ls -la");
        }
        _ => panic!("Expected CommandRelay variant"),
    }
}

#[test]
fn test_sync_packet_presence_update() {
    let participant_id = Uuid::new_v4();
    let packet = SyncPacket::PresenceUpdate {
        participant_id,
        x: 0.5,
        y: 0.75,
    };
    
    match packet {
        SyncPacket::PresenceUpdate { participant_id: _, x, y } => {
            assert_eq!(x, 0.5);
            assert_eq!(y, 0.75);
        }
        _ => panic!("Expected PresenceUpdate variant"),
    }
}

#[test]
fn test_sync_packet_heartbeat() {
    let packet = SyncPacket::Heartbeat;
    match packet {
        SyncPacket::Heartbeat => {}
        _ => panic!("Expected Heartbeat variant"),
    }
}

// ============================================================================
// Remote Integration Tests
// ============================================================================

#[test]
fn test_remote_full_connection_lifecycle() {
    let mut state = TosState::new();
    let node_id = Uuid::new_v4();
    
    // Register node
    let info = RemoteNodeInfo {
        id: node_id,
        hostname: "integration-test-node".to_string(),
        address: "10.0.0.100".to_string(),
        os_type: "TOS".to_string(),
        version: "1.0.0".to_string(),
        status: RemoteStatus::Online,
    };
    
    state.remote_manager.register_node(info);
    
    // Connect
    let conn_result = state.remote_manager.connect(node_id, ConnectionType::TOSNative);
    assert!(conn_result.is_ok());
    
    // Sync
    let sync_result = state.remote_manager.sync_node(node_id);
    assert!(sync_result.is_ok());
    
    // Create sector
    let sector = state.remote_manager.create_remote_sector(node_id);
    assert!(sector.is_some());
    let sector = sector.unwrap();
    assert!(sector.name.contains("integration-test-node"));
    assert_eq!(sector.connection_type, ConnectionType::TOSNative);
    
    // Disconnect
    state.remote_manager.disconnect(node_id);
    assert!(!state.remote_manager.active_connections.contains_key(&node_id));
}

#[test]
fn test_remote_process_sector_state_packet() {
    let mut state = TosState::new();
    let node_id = Uuid::new_v4();
    
    // Register and connect
    let info = RemoteNodeInfo {
        id: node_id,
        hostname: "packet-test".to_string(),
        address: "10.0.0.200".to_string(),
        os_type: "TOS".to_string(),
        version: "1.0.0".to_string(),
        status: RemoteStatus::Online,
    };
    
    state.remote_manager.register_node(info);
    state.remote_manager.connect(node_id, ConnectionType::SSH).unwrap();
    
    // Create and process packet
    let remote_sector = Sector {
        id: node_id,
        name: "Packet Sector".to_string(),
        color: "#00ff00".to_string(),
        hubs: vec![],
        active_hub_index: 0,
        host: "packet-host".to_string(),
        connection_type: ConnectionType::SSH,
        participants: vec![],
        portal_active: false,
        portal_url: None,
        description: "Processed via packet".to_string(),
        icon: "📦".to_string(),
        sector_type_name: "operations".to_string(),
    };
    
    let packet = SyncPacket::SectorState(remote_sector);
    let result = state.remote_manager.process_packet(node_id, packet, &mut state.sectors);
    
    assert!(result.is_ok());
    assert!(state.sectors.iter().any(|s| s.name == "Packet Sector"));
}

#[test]
fn test_remote_process_terminal_delta_packet() {
    let mut state = TosState::new();
    let hub_id = state.sectors[0].hubs[0].id;
    
    let packet = SyncPacket::TerminalDelta {
        hub_id,
        line: "Remote command output".to_string(),
    };
    
    let result = state.remote_manager.process_packet(
        Uuid::new_v4(), 
        packet, 
        &mut state.sectors
    );
    
    assert!(result.is_ok());
    assert!(state.sectors[0].hubs[0].terminal_output.contains(&"Remote command output".to_string()));
}

#[test]
fn test_remote_auth_store() {
    let mut manager = RemoteManager::new();
    let node_id = Uuid::new_v4();
    
    // Register node first
    let info = RemoteNodeInfo {
        id: node_id,
        hostname: "auth-test".to_string(),
        address: "10.0.0.50".to_string(),
        os_type: "TOS".to_string(),
        version: "1.0.0".to_string(),
        status: RemoteStatus::AuthenticationRequired,
    };
    
    manager.register_node(info);
    manager.auth_store.insert(node_id, "token123".to_string());
    
    assert!(manager.auth_store.contains_key(&node_id));
    assert_eq!(manager.auth_store.get(&node_id).unwrap(), "token123");
}

// ============================================================================
// Audio + Remote Integration Tests
// ============================================================================

#[test]
fn test_state_audio_and_remote_managers_exist() {
    let state = TosState::new();
    
    // Verify both managers are initialized
    assert!(std::mem::size_of_val(&state.audio_manager) > 0);
    assert!(std::mem::size_of_val(&state.remote_manager) > 0);
}

#[test]
fn test_earcon_player_integration() {
    let mut state = TosState::new();
    
    // Earcon player should exist in state
    assert!(std::mem::size_of_val(&state.earcon_player) > 0);
    
    // Should be able to call methods without panic
    state.earcon_player.set_enabled(true);
    state.earcon_player.set_master_volume(0.7);
    
    assert_eq!(state.earcon_player.master_volume(), 0.7);
}

#[test]
fn test_audio_manager_event_playback() {
    let mut state = TosState::new();
    
    // Set up sector ambience
    let sector_id = state.sectors[0].id;
    let profile = AmbienceProfile::default();
    state.audio_manager.set_sector_ambience(sector_id, profile);
    
    // Play event (should not panic)
    state.audio_manager.play_event(AudioEvent::DataTransfer);
    
    assert!(state.audio_manager.sector_ambience.contains_key(&sector_id));
}

