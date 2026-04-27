use tos_common::platform::{InputSource, RawInputEvent, SemanticEvent};
use tos_common::platform::quest::QuestInput;
use tos_common::DeviceMapping;

#[test]
fn test_controller_button_mapping() {
    let input = QuestInput::default();
    
    // Test South button -> Select
    let raw = RawInputEvent::ControllerButtonDown {
        controller_id: 0,
        button: "South".to_string(),
    };
    let semantic = input.map_to_semantic(raw).expect("Should map South button");
    match semantic {
        SemanticEvent::Select(target) => assert_eq!(target, "default"),
        _ => panic!("Expected Select event"),
    }

    // Test North button -> ZoomOut
    let raw = RawInputEvent::ControllerButtonDown {
        controller_id: 0,
        button: "North".to_string(),
    };
    let semantic = input.map_to_semantic(raw).expect("Should map North button");
    assert!(matches!(semantic, SemanticEvent::ZoomOut));
}

#[test]
fn test_spatial_gesture_mapping() {
    let input = QuestInput::default();
    
    // Test pinch_left -> ZoomOut
    let raw = RawInputEvent::SpatialGesture {
        hand: "left".to_string(),
        gesture: "pinch_left".to_string(),
    };
    let semantic = input.map_to_semantic(raw).expect("Should map pinch_left");
    assert!(matches!(semantic, SemanticEvent::ZoomOut));

    // Test pinch_right -> ZoomIn
    let raw = RawInputEvent::SpatialGesture {
        hand: "right".to_string(),
        gesture: "pinch_right".to_string(),
    };
    let semantic = input.map_to_semantic(raw).expect("Should map pinch_right");
    assert!(matches!(semantic, SemanticEvent::ZoomIn));
}

#[test]
fn test_gaze_dwell_mapping() {
    let input = QuestInput::default();
    
    // Test gaze dwell < 500ms -> None
    let raw = RawInputEvent::SpatialGaze {
        x: 0.0,
        y: 0.0,
        z: -1.0,
        dwell_ms: 400,
    };
    assert!(input.map_to_semantic(raw).is_none());

    // Test gaze dwell >= 500ms -> Select
    let raw = RawInputEvent::SpatialGaze {
        x: 0.0,
        y: 0.0,
        z: -1.0,
        dwell_ms: 500,
    };
    let semantic = input.map_to_semantic(raw).expect("Should map gaze dwell");
    match semantic {
        SemanticEvent::Select(target) => assert_eq!(target, "gaze_target"),
        _ => panic!("Expected Select event"),
    }
}

#[test]
fn test_custom_mapping() {
    let mut mapping = DeviceMapping::default();
    mapping.button_mappings.insert("South".to_string(), "zoom_in".to_string());
    
    let input = QuestInput { mapping };
    
    let raw = RawInputEvent::ControllerButtonDown {
        controller_id: 0,
        button: "South".to_string(),
    };
    let semantic = input.map_to_semantic(raw).expect("Should map South button");
    assert!(matches!(semantic, SemanticEvent::ZoomIn));
}
