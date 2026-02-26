use tos_core::TosState;
use tos_core::system::input::SemanticEvent;
use tos_core::system::input::advanced::{AdvancedInputEvent, DeviceId, InputDeviceType, HandGesture, Handedness};

#[test]
fn test_voice_mapping_ai() {
    let mut state = TosState::new_fresh();
    
    // Test AI mode toggle
    let cmd = state.voice.process_text("ai mode").expect("Should parse ai mode");
    assert_eq!(cmd.event, SemanticEvent::AiModeToggle);
    
    // Test AI submit
    let cmd = state.voice.process_text("submit").expect("Should parse submit");
    assert_eq!(cmd.event, SemanticEvent::AiSubmit);
    
    // Test AI stop
    let cmd = state.voice.process_text("stop").expect("Should parse stop");
    assert_eq!(cmd.event, SemanticEvent::AiStop);
}

#[test]
fn test_advanced_input_gesture_mapping() {
    let mut state = TosState::new_fresh();
    
    let device = DeviceId::new(InputDeviceType::HandTracking, 0);
    
    state.handle_semantic_event(SemanticEvent::ZoomIn);
    assert_eq!(state.current_level, tos_core::HierarchyLevel::CommandHub);

    // Peace sign -> CycleMode
    let event = AdvancedInputEvent::Gesture {
        device: device.clone(),
        gesture: HandGesture::PeaceSign,
        hand: Handedness::Right,
    };
    
    state.handle_advanced_input(event);
    
    // Current mode should have cycled from Command to Directory
    let viewport = &state.viewports[0];
    let current_mode = state.sectors[viewport.sector_index].hubs[viewport.hub_index].mode;
    assert_eq!(current_mode, tos_core::CommandHubMode::Directory);
}

#[test]
fn test_advanced_input_vr_zoom() {
    let mut state = TosState::new_fresh();
    let initial_level = state.current_level;
    
    let device = DeviceId::new(InputDeviceType::VRController, 0);
    
    // Thumbstick up -> ZoomIn
    let event = AdvancedInputEvent::VRController {
        device: device.clone(),
        state: tos_core::system::input::advanced::VRControllerState {
            thumbstick: (0.0, 0.9), // Up
            ..Default::default()
        },
        delta: Default::default(),
    };
    
    state.handle_advanced_input(event);
    
    // Level should have changed (Zoomed in from GlobalOverview to CommandHub)
    assert_ne!(state.current_level, initial_level);
    assert_eq!(state.current_level, tos_core::HierarchyLevel::CommandHub);
}
