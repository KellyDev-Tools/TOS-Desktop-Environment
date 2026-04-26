use tos_common::state::*;
use tos_common::ipc::*;
use tos_common::collaboration::*;
use tos_common::marketplace::*;
use uuid::Uuid;
use chrono::Local;

#[test]
fn test_hierarchy_level_serde() {
    let level = HierarchyLevel::CommandHub;
    let serialized = serde_json::to_string(&level).unwrap();
    assert_eq!(serialized, "\"CommandHub\"");
    let deserialized: HierarchyLevel = serde_json::from_str(&serialized).unwrap();
    assert_eq!(level, deserialized);
}

#[test]
fn test_semantic_event_serde() {
    let event = SemanticEvent::PromptSubmit("hello".to_string());
    let serialized = serde_json::to_string(&event).unwrap();
    let deserialized: SemanticEvent = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        SemanticEvent::PromptSubmit(s) => assert_eq!(s, "hello"),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_terminal_line_serde() {
    let line = TerminalLine {
        text: "System boot".to_string(),
        priority: 3,
        timestamp: Local::now(),
    };
    let serialized = serde_json::to_string(&line).unwrap();
    let deserialized: TerminalLine = serde_json::from_str(&serialized).unwrap();
    assert_eq!(line.text, deserialized.text);
    assert_eq!(line.priority, deserialized.priority);
}

#[test]
fn test_face_register_serde() {
    let reg = FaceRegister {
        face_id: Uuid::new_v4(),
        profile: FaceProfile::Desktop,
        version: "0.1.0".to_string(),
    };
    let serialized = serde_json::to_string(&reg).unwrap();
    let deserialized: FaceRegister = serde_json::from_str(&serialized).unwrap();
    assert_eq!(reg.face_id, deserialized.face_id);
    assert_eq!(reg.profile, deserialized.profile);
}

#[test]
fn test_webrtc_payload_serde() {
    let payload = WebRtcPayload::CursorSync {
        user: Uuid::new_v4(),
        x: 0.5,
        y: 0.5,
        target: Some("viewport-1".to_string()),
    };
    let serialized = serde_json::to_string(&payload).unwrap();
    let deserialized: WebRtcPayload = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        WebRtcPayload::CursorSync { x, y, .. } => {
            assert_eq!(x, 0.5);
            assert_eq!(y, 0.5);
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_marketplace_summary_serde() {
    let summary = MarketplaceModuleSummary {
        id: "mod-1".to_string(),
        name: "Cool Module".to_string(),
        module_type: "terminal".to_string(),
        author: "Tim".to_string(),
        icon: None,
        rating: 4.5,
        price: "Free".to_string(),
        installed: true,
    };
    let serialized = serde_json::to_string(&summary).unwrap();
    let deserialized: MarketplaceModuleSummary = serde_json::from_str(&serialized).unwrap();
    assert_eq!(summary.id, deserialized.id);
    assert_eq!(summary.rating, deserialized.rating);
}

#[test]
fn test_settings_store_secure_skip_serializing() {
    use std::collections::HashMap;
    let mut secure = HashMap::new();
    secure.insert("api_key".to_string(), "SECRET_VALUE".to_string());
    
    let store = SettingsStore {
        global: HashMap::new(),
        sectors: HashMap::new(),
        applications: HashMap::new(),
        ai_patterns: HashMap::new(),
        secure,
    };
    
    let serialized = serde_json::to_string(&store).unwrap();
    
    // Ensure the secret value is NOT in the serialized string
    assert!(!serialized.contains("SECRET_VALUE"), "Sensitive data leaked in serialization!");
    assert!(!serialized.contains("\"secure\""), "Secure field name should be skipped");
    
    // Ensure deserialization still works (it will just have an empty map due to #[serde(default)])
    let deserialized: SettingsStore = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.secure.is_empty());
}
