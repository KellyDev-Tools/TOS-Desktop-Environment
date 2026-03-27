//! Live Feed Tests Implementation
//! 
//! Tests for WebSocket streaming, test recording, and
//! real-time state broadcasting to the TOS Web Portal.

#![cfg(feature = "live-feed")]

use tos_core::system::live_feed::*;
use tokio::sync::mpsc;

#[test]
fn test_live_feed_config_default() {
    let config = LiveFeedConfig::default();
    
    assert_eq!(config.port, 8765);
    assert_eq!(config.bind_address, "0.0.0.0");
    assert!(config.enable_recording);
    assert_eq!(config.update_frequency, 30.0);
    assert_eq!(config.max_clients, 10);
    assert!(config.compression);
}

#[test]
fn test_live_feed_config_custom() {
    let config = LiveFeedConfig {
        port: 9000,
        bind_address: "127.0.0.1".to_string(),
        update_frequency: 60.0,
        max_clients: 5,
        auth_token: Some("secret123".to_string()),
        ..Default::default()
    };
    
    assert_eq!(config.port, 9000);
    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.update_frequency, 60.0);
    assert_eq!(config.max_clients, 5);
    assert_eq!(config.auth_token, Some("secret123".to_string()));
}

#[test]
fn test_state_snapshot_creation() {
    use tos_core::TosState;
    
    let state = TosState::new();
    let snapshot = create_test_snapshot(&state);
    
    assert_eq!(snapshot.sectors.len(), 3);
    assert!(snapshot.fps > 0.0);
    assert!(!snapshot.current_level.is_empty());
}

#[test]
fn test_test_event_types() {
    let events = vec![
        TestEventType::Started,
        TestEventType::Passed,
        TestEventType::Failed,
        TestEventType::Skipped,
        TestEventType::InProgress,
        TestEventType::Assertion,
        TestEventType::Error,
        TestEventType::Completed,
    ];
    
    // All event types should be serializable
    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_test_event_serialization() {
    let event = TestEvent {
        test_id: "test_001".to_string(),
        test_name: "Zoom Navigation Test".to_string(),
        event_type: TestEventType::Passed,
        timestamp: 1234567890,
        details: serde_json::json!({
            "steps": 3,
            "duration_ms": 1500,
            "assertions": 5
        }),
        screenshot: None,
    };
    
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("test_001"));
    assert!(json.contains("Zoom Navigation Test"));
    assert!(json.contains("Passed"));
    
    // Deserialize
    let deserialized: TestEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.test_id, "test_001");
    assert_eq!(deserialized.event_type, TestEventType::Passed);
}

#[test]
fn test_performance_data_serialization() {
    let data = PerformanceData {
        timestamp: 1234567890,
        fps: 59.5,
        frame_time_ms: 16.8,
        cpu_usage: 25.5,
        memory_mb: 512.0,
        gpu_usage: Some(45.0),
        render_time_ms: 12.5,
    };
    
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("59.5"));
    assert!(json.contains("16.8"));
    
    let deserialized: PerformanceData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.fps, 59.5);
    assert_eq!(deserialized.gpu_usage, Some(45.0));
}

#[test]
fn test_accessibility_event_data() {
    let event = AccessibilityEventData {
        timestamp: 1234567890,
        event_type: "navigation".to_string(),
        description: "Zoomed to Command Hub".to_string(),
        aria_live: "polite".to_string(),
    };
    
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("navigation"));
    assert!(json.contains("Command Hub"));
    assert!(json.contains("polite"));
}

#[test]
fn test_interaction_data() {
    let interaction = InteractionData {
        timestamp: 1234567890,
        input_type: "keyboard".to_string(),
        semantic_event: "ZoomIn".to_string(),
        target_element: Some("sector-1".to_string()),
        coordinates: Some((100.0, 200.0)),
    };
    
    let json = serde_json::to_string(&interaction).unwrap();
    assert!(json.contains("keyboard"));
    assert!(json.contains("ZoomIn"));
    assert!(json.contains("sector-1"));
}

#[test]
fn test_notification_data() {
    let notification = NotificationData {
        timestamp: 1234567890,
        level: "warning".to_string(),
        message: "Performance alert: FPS dropped".to_string(),
        source: "performance_monitor".to_string(),
    };
    
    let json = serde_json::to_string(&notification).unwrap();
    assert!(json.contains("warning"));
    assert!(json.contains("FPS dropped"));
}

#[test]
fn test_live_feed_message_variants() {
    // Test all message variants can be serialized
    let messages = vec![
        LiveFeedMessage::Ping,
        LiveFeedMessage::Pong,
        LiveFeedMessage::AuthRequired,
        LiveFeedMessage::AuthSuccess,
        LiveFeedMessage::Error("Test error".to_string()),
    ];
    
    for msg in messages {
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.is_empty());
        
        // Verify tag is present
        let deserialized: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(deserialized.get("type").is_some());
    }
}

#[test]
fn test_state_change_structure() {
    let change = StateChange {
        path: "sectors.0.hubs.0.mode".to_string(),
        old_value: Some(serde_json::json!("Command")),
        new_value: serde_json::json!("Directory"),
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("sectors.0.hubs.0.mode"));
    assert!(json.contains("Command"));
    assert!(json.contains("Directory"));
}

#[test]
fn test_sector_snapshot_structure() {
    let sector = SectorSnapshot {
        id: "uuid-123".to_string(),
        name: "Alpha Sector".to_string(),
        color: "#ff9900".to_string(),
        host: "LOCAL".to_string(),
        connection_type: "Local".to_string(),
        portal_active: false,
        portal_url: None,
        participant_count: 1,
        hubs: vec![],
        active_hub_index: 0,
    };
    
    let json = serde_json::to_string(&sector).unwrap();
    assert!(json.contains("Alpha Sector"));
    assert!(json.contains("#ff9900"));
    assert!(json.contains("LOCAL"));
}

#[test]
fn test_hub_snapshot_structure() {
    let hub = HubSnapshot {
        id: "hub-123".to_string(),
        mode: "Command".to_string(),
        prompt: "ls -la".to_string(),
        application_count: 2,
        active_app_index: Some(0),
        confirmation_required: Some("rm -rf /".to_string()),
    };
    
    let json = serde_json::to_string(&hub).unwrap();
    assert!(json.contains("Command"));
    assert!(json.contains("ls -la"));
    assert!(json.contains("rm -rf /"));
}

#[test]
fn test_viewport_snapshot_structure() {
    let viewport = ViewportSnapshot {
        id: "vp-123".to_string(),
        sector_index: 0,
        hub_index: 0,
        current_level: "CommandHub".to_string(),
        bezel_expanded: true,
    };
    
    let json = serde_json::to_string(&viewport).unwrap();
    assert!(json.contains("CommandHub"));
    assert!(json.contains("bezel_expanded"));
}

#[test]
fn test_tos_state_snapshot_full() {
    use tos_core::TosState;
    
    let state = TosState::new();
    let snapshot = create_test_snapshot(&state);
    
    // Verify snapshot structure
    assert!(snapshot.timestamp != 0);
    assert_eq!(snapshot.sectors.len(), state.sectors.len());
    assert_eq!(snapshot.viewports.len(), state.viewports.len());
    
    // Check first sector
    let first_sector = &snapshot.sectors[0];
    assert_eq!(first_sector.name, "Alpha Sector");
    assert_eq!(first_sector.color, "#ff9900");
    
    // Check first viewport
    let first_viewport = &snapshot.viewports[0];
    assert_eq!(first_viewport.sector_index, 0);
    assert_eq!(first_viewport.hub_index, 0);
}

#[test]
fn test_feed_command_variants() {
    // Test that all command variants can be created
    let _start = FeedCommand::StartRecording("test_name".to_string());
    let _stop = FeedCommand::StopRecording;
    let _pause = FeedCommand::PauseStreaming;
    let _resume = FeedCommand::ResumeStreaming;
    let _auth = FeedCommand::Authenticate(
        "127.0.0.1:1234".parse().unwrap(),
        "token123".to_string()
    );
    let _disconnect = FeedCommand::DisconnectClient(
        "127.0.0.1:1234".parse().unwrap()
    );
    let _shutdown = FeedCommand::Shutdown;
}

// Helper function to create a test snapshot
fn create_test_snapshot(state: &tos_core::TosState) -> TosStateSnapshot {
    TosStateSnapshot {
        timestamp: 1234567890,
        current_level: format!("{:?}", state.current_level),
        sectors: state.sectors.iter().map(|s| SectorSnapshot {
            id: s.id.to_string(),
            name: s.name.clone(),
            color: s.color.clone(),
            host: s.host.clone(),
            connection_type: format!("{:?}", s.connection_type),
            portal_active: s.portal_active,
            portal_url: s.portal_url.clone(),
            participant_count: s.participants.len(),
            hubs: s.hubs.iter().map(|h| HubSnapshot {
                id: h.id.to_string(),
                mode: format!("{:?}", h.mode),
                prompt: h.prompt.clone(),
                application_count: h.applications.len(),
                active_app_index: h.active_app_index,
                confirmation_required: h.confirmation_required.clone(),
            }).collect(),
            active_hub_index: s.active_hub_index,
        }).collect(),
        active_viewport: state.active_viewport_index,
        fps: state.fps,
        performance_alert: state.performance_alert,
        viewports: state.viewports.iter().map(|v| ViewportSnapshot {
            id: v.id.to_string(),
            sector_index: v.sector_index,
            hub_index: v.hub_index,
            current_level: format!("{:?}", v.current_level),
            bezel_expanded: v.bezel_expanded,
        }).collect(),
    }
}

#[tokio::test]
async fn test_live_feed_server_creation() {
    let config = LiveFeedConfig::default();
    let server = LiveFeedServer::new(config);
    
    // Verify server was created
    assert!(!server.is_recording().await);
    assert!(server.command_sender().capacity() > 0);
}

#[tokio::test]
async fn test_recording_lifecycle() {
    let config = LiveFeedConfig {
        enable_recording: true,
        recording_path: "/tmp/test-recordings".to_string(),
        ..Default::default()
    };
    
    let server = LiveFeedServer::new(config);
    
    // Initially not recording
    assert!(!server.is_recording().await);
    let info: Option<(String, String, std::time::Duration)> = server.recording_info().await;
    assert!(info.is_none());
    
    // Start recording directly (bypassing the channel for testing)
    server.process_command(FeedCommand::StartRecording("test_lifecycle".to_string())).await;
    
    // Should be recording now
    assert!(server.is_recording().await);
    let info: Option<(String, String, std::time::Duration)> = server.recording_info().await;
    assert!(info.is_some());
    
    let (test_id, test_name, _duration) = info.unwrap();
    assert!(test_id.starts_with("test_"));
    assert_eq!(test_name, "test_lifecycle");
    
    // Stop recording directly
    server.process_command(FeedCommand::StopRecording).await;
    
    // Should not be recording anymore
    assert!(!server.is_recording().await);
}

#[test]
fn test_test_event_type_serialization() {
    // Test that all variants serialize correctly
    let types = vec![
        (TestEventType::Started, "\"Started\""),
        (TestEventType::Passed, "\"Passed\""),
        (TestEventType::Failed, "\"Failed\""),
        (TestEventType::Skipped, "\"Skipped\""),
        (TestEventType::InProgress, "\"InProgress\""),
        (TestEventType::Assertion, "\"Assertion\""),
        (TestEventType::Error, "\"Error\""),
        (TestEventType::Completed, "\"Completed\""),
    ];
    
    for (event_type, expected) in types {
        let json = serde_json::to_string(&event_type).unwrap();
        assert_eq!(json, expected);
    }
}

#[test]
fn test_live_feed_message_with_data() {
    // Test StateSnapshot message
    let state = tos_core::TosState::new();
    let snapshot = create_test_snapshot(&state);
    let msg = LiveFeedMessage::StateSnapshot(snapshot);
    
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized["type"], "StateSnapshot");
    assert!(deserialized["data"].is_object());
}

#[test]
fn test_performance_data_bounds() {
    // Test reasonable performance bounds
    let data = PerformanceData {
        timestamp: 0,
        fps: 144.0, // High refresh rate
        frame_time_ms: 6.94,
        cpu_usage: 100.0, // Max CPU
        memory_mb: 16384.0, // 16GB
        gpu_usage: Some(100.0),
        render_time_ms: 3.0,
    };
    
    assert!(data.fps > 0.0);
    assert!(data.frame_time_ms > 0.0);
    assert!(data.cpu_usage >= 0.0 && data.cpu_usage <= 100.0);
    assert!(data.memory_mb > 0.0);
}

#[test]
fn test_accessibility_event_aria_levels() {
    let levels = vec!["off", "polite", "assertive"];
    
    for level in levels {
        let event = AccessibilityEventData {
            timestamp: 0,
            event_type: "test".to_string(),
            description: "Test".to_string(),
            aria_live: level.to_string(),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(level));
    }
}
