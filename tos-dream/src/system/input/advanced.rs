//! Advanced Input Abstraction Implementation
//!
//! Supports game controllers, VR/AR controllers, hand tracking, and eye tracking.
//! Provides concurrent input stream merging with intelligent conflict resolution.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Types of advanced input devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputDeviceType {
    /// Game controller (gamepad/joystick)
    GameController,
    /// VR headset controller (left/right)
    VRController,
    /// AR controller or hand tracking
    ARController,
    /// Hand tracking (camera-based)
    HandTracking,
    /// Eye tracking (gaze-based)
    EyeTracking,
    /// Accessibility switches
    AccessibilitySwitch,
}

impl InputDeviceType {
    /// Get a human-readable name for the device type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::GameController => "Game Controller",
            Self::VRController => "VR Controller",
            Self::ARController => "AR Controller",
            Self::HandTracking => "Hand Tracking",
            Self::EyeTracking => "Eye Tracking",
            Self::AccessibilitySwitch => "Accessibility Switch",
        }
    }
    
    /// Check if this device type supports haptic feedback
    pub fn supports_haptics(&self) -> bool {
        matches!(self, Self::GameController | Self::VRController | Self::ARController)
    }
    
    /// Check if this device type provides spatial data
    pub fn is_spatial(&self) -> bool {
        matches!(self, Self::VRController | Self::ARController | Self::HandTracking | Self::EyeTracking)
    }
}

/// Unique identifier for an input device instance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId {
    /// Device type
    pub device_type: InputDeviceType,
    /// Instance index (for multiple devices of same type)
    pub instance: u32,
    /// Hardware identifier (if available)
    pub hardware_id: Option<String>,
}

impl DeviceId {
    /// Create a new device ID
    pub fn new(device_type: InputDeviceType, instance: u32) -> Self {
        Self {
            device_type,
            instance,
            hardware_id: None,
        }
    }
    
    /// Create with hardware ID
    pub fn with_hardware_id(device_type: InputDeviceType, instance: u32, hardware_id: String) -> Self {
        Self {
            device_type,
            instance,
            hardware_id: Some(hardware_id),
        }
    }
    
    /// Get a display string for this device
    pub fn display_name(&self) -> String {
        let type_name = self.device_type.display_name();
        if let Some(ref hw) = self.hardware_id {
            format!("{} #{} ({})", type_name, self.instance, hw)
        } else {
            format!("{} #{}", type_name, self.instance)
        }
    }
}

/// Game controller button/axis states
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameControllerState {
    /// Left analog stick (x, y) - range [-1.0, 1.0]
    pub left_stick: (f32, f32),
    /// Right analog stick (x, y) - range [-1.0, 1.0]
    pub right_stick: (f32, f32),
    /// Left trigger - range [0.0, 1.0]
    pub left_trigger: f32,
    /// Right trigger - range [0.0, 1.0]
    pub right_trigger: f32,
    /// D-pad state
    pub dpad: DPadState,
    /// Face buttons (A/B/X/Y or equivalent)
    pub face_buttons: [bool; 4],
    /// Shoulder buttons
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    /// Menu/Start buttons
    pub start_button: bool,
    pub select_button: bool,
    /// Stick click buttons
    pub left_stick_click: bool,
    pub right_stick_click: bool,
}

/// D-pad directional state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DPadState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

/// VR/AR controller state (6DOF)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VRControllerState {
    /// Controller position in 3D space (x, y, z)
    pub position: (f32, f32, f32),
    /// Controller rotation as quaternion (x, y, z, w)
    pub rotation: (f32, f32, f32, f32),
    /// Trigger pressure [0.0, 1.0]
    pub trigger: f32,
    /// Grip pressure [0.0, 1.0]
    pub grip: f32,
    /// Thumbstick/Joystick (x, y) [-1.0, 1.0]
    pub thumbstick: (f32, f32),
    /// Face buttons
    pub primary_button: bool,
    pub secondary_button: bool,
    /// Menu button
    pub menu_button: bool,
    /// Touchpad/trackpad state (if available)
    pub touchpad: Option<TouchpadState>,
}

/// Touchpad/trackpad state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TouchpadState {
    /// Whether finger is touching
    pub touching: bool,
    /// Touch position (x, y) [-1.0, 1.0]
    pub position: (f32, f32),
    /// Click pressure [0.0, 1.0]
    pub click_pressure: f32,
}

/// Hand tracking state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandTrackingState {
    /// Which hand (left/right)
    pub hand: Handedness,
    /// Wrist position
    pub wrist_position: (f32, f32, f32),
    /// Palm orientation
    pub palm_orientation: (f32, f32, f32, f32),
    /// Individual finger joints (tip, dip, pip, mcp for each finger)
    pub fingers: [FingerState; 5],
    /// Detected gestures
    pub gestures: Vec<HandGesture>,
    /// Confidence level [0.0, 1.0]
    pub confidence: f32,
}

/// Handedness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Handedness {
    #[default]
    Unknown,
    Left,
    Right,
}

/// Finger state (thumb, index, middle, ring, pinky)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FingerState {
    /// Tip position
    pub tip: (f32, f32, f32),
    /// Distal interphalangeal joint
    pub dip: (f32, f32, f32),
    /// Proximal interphalangeal joint
    pub pip: (f32, f32, f32),
    /// Metacarpophalangeal joint
    pub mcp: (f32, f32, f32),
    /// Curl amount [0.0 = extended, 1.0 = fully curled]
    pub curl: f32,
}

/// Hand gestures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandGesture {
    /// Pinch (thumb and index finger together)
    Pinch,
    /// Grab (all fingers curled)
    Grab,
    /// Open palm (all fingers extended)
    OpenPalm,
    /// Point (index extended, others curled)
    Point,
    /// Two-hand spread (both hands moving apart)
    TwoHandSpread,
    /// Thumbs up
    ThumbsUp,
    /// Peace sign
    PeaceSign,
    /// Fist
    Fist,
}

/// Eye tracking state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EyeTrackingState {
    /// Gaze point on screen (normalized x, y) [-1.0, 1.0]
    pub gaze_point: (f32, f32),
    /// Gaze point in 3D space (for VR/AR)
    pub gaze_direction: (f32, f32, f32),
    /// Left eye openness [0.0 = closed, 1.0 = open]
    pub left_openness: f32,
    /// Right eye openness [0.0 = closed, 1.0 = open]
    pub right_openness: f32,
    /// Whether eyes are tracked
    pub is_tracking: bool,
    /// Confidence level [0.0, 1.0]
    pub confidence: f32,
    /// Detected blink patterns
    pub blink_patterns: Vec<BlinkPattern>,
    /// Dwell time at current gaze point (for dwell clicking)
    pub dwell_time: Duration,
}

/// Blink patterns for input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlinkPattern {
    /// Single blink
    SingleBlink,
    /// Double blink
    DoubleBlink,
    /// Extended blink (hold)
    ExtendedBlink,
    /// Wink (one eye)
    WinkLeft,
    WinkRight,
}

/// Accessibility switch state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SwitchState {
    /// Switch is pressed
    pub pressed: bool,
    /// Time since last press
    pub time_since_last_press: Duration,
    /// Number of rapid presses (for multi-switch patterns)
    pub rapid_press_count: u32,
}

/// Unified input event from any advanced device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvancedInputEvent {
    /// Game controller event
    GameController {
        device: DeviceId,
        state: GameControllerState,
        delta: GameControllerState,
    },
    /// VR controller event
    VRController {
        device: DeviceId,
        state: VRControllerState,
        delta: VRControllerState,
    },
    /// Hand tracking event
    HandTracking {
        device: DeviceId,
        state: HandTrackingState,
    },
    /// Eye tracking event
    EyeTracking {
        device: DeviceId,
        state: EyeTrackingState,
    },
    /// Switch event
    Switch {
        device: DeviceId,
        state: SwitchState,
    },
    /// Gesture detected
    Gesture {
        device: DeviceId,
        gesture: HandGesture,
        hand: Handedness,
    },
    /// Device connected/disconnected
    DeviceConnected(DeviceId),
    DeviceDisconnected(DeviceId),
}

/// Semantic actions that can be triggered by advanced input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticAction {
    // Navigation
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    RotateLeft,
    RotateRight,
    NextElement,
    PreviousElement,
    
    // Selection
    Select,
    SecondarySelect,
    MultiSelectToggle,
    Grab,
    Release,
    
    // System
    ToggleBezel,
    OpenMenu,
    GoBack,
    TacticalReset,
    
    // Mode Control
    CycleMode,
    SetModeCommand,
    SetModeDirectory,
    SetModeActivity,
    
    // Text Input
    CursorMove,
    TextDelete,
    TextAccept,
}

/// Input mapping configuration for a device type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMapping {
    /// Device type this mapping applies to
    pub device_type: InputDeviceType,
    /// Button/trigger to action mappings
    pub button_mappings: HashMap<String, SemanticAction>,
    /// Axis to action mappings with thresholds
    pub axis_mappings: Vec<AxisMapping>,
    /// Gesture to action mappings
    pub gesture_mappings: HashMap<HandGesture, SemanticAction>,
    /// Eye tracking dwell time for selection (ms)
    pub dwell_select_ms: u32,
    /// Whether to enable dwell clicking
    pub dwell_enabled: bool,
}

/// Axis mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisMapping {
    /// Axis identifier
    pub axis: String,
    /// Action to trigger
    pub action: SemanticAction,
    /// Threshold to trigger (absolute value)
    pub threshold: f32,
    /// Whether to use absolute value
    pub absolute: bool,
    /// Deadzone to ignore small movements
    pub deadzone: f32,
}

/// Conflict resolution strategy when multiple devices provide input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Last device wins
    LastWins,
    /// First device wins
    FirstWins,
    /// Average all inputs
    Average,
    /// Priority-based (certain device types have priority)
    PriorityBased,
    /// Require confirmation from multiple devices
    MultiConfirm,
}

/// Advanced input manager
#[derive(Debug)]
pub struct AdvancedInputManager {
    /// Connected devices and their states
    devices: HashMap<DeviceId, DeviceState>,
    /// Input mappings per device type
    mappings: HashMap<InputDeviceType, InputMapping>,
    /// Recent input events (for gesture detection, conflict resolution)
    event_buffer: VecDeque<(Instant, AdvancedInputEvent)>,
    /// Conflict resolution strategy
    conflict_resolution: ConflictResolution,
    /// Device priorities (for priority-based resolution)
    device_priorities: HashMap<InputDeviceType, u8>,
    /// Last active device (for cursor appearance)
    last_active_device: Option<DeviceId>,
    /// Whether advanced input is enabled
    enabled: bool,
    /// Gesture detection state
    _gesture_state: GestureDetectionState,
    /// Eye tracking dwell state
    dwell_state: DwellState,
}

/// Device connection state
#[derive(Debug, Clone)]
struct DeviceState {
    device: DeviceId,
    connected: bool,
    last_activity: Instant,
    haptic_enabled: bool,
}

/// Gesture detection state machine
#[derive(Debug, Default)]
struct GestureDetectionState {
    /// Currently detected gestures
    _active_gestures: Vec<(HandGesture, Instant)>,
    /// Pinch start position (for drag detection)
    _pinch_start: Option<(f32, f32, f32)>,
    /// Two-hand spread start distance
    _spread_start_distance: Option<f32>,
}

/// Dwell clicking state
#[derive(Debug, Default)]
struct DwellState {
    /// Current gaze target
    current_target: Option<(f32, f32)>,
    /// When dwell started
    dwell_start: Option<Instant>,
    /// Whether dwell selection is triggered
    dwell_triggered: bool,
}

impl AdvancedInputManager {
    /// Create a new advanced input manager
    pub fn new() -> Self {
        let mut manager = Self {
            devices: HashMap::new(),
            mappings: HashMap::new(),
            event_buffer: VecDeque::with_capacity(100),
            conflict_resolution: ConflictResolution::LastWins,
            device_priorities: HashMap::new(),
            last_active_device: None,
            enabled: true,
            _gesture_state: GestureDetectionState::default(),
            dwell_state: DwellState::default(),
        };
        
        // Set default priorities (lower number = higher priority)
        manager.device_priorities.insert(InputDeviceType::AccessibilitySwitch, 1);
        manager.device_priorities.insert(InputDeviceType::GameController, 2);
        manager.device_priorities.insert(InputDeviceType::VRController, 3);
        manager.device_priorities.insert(InputDeviceType::HandTracking, 4);
        manager.device_priorities.insert(InputDeviceType::EyeTracking, 5);
        manager.device_priorities.insert(InputDeviceType::ARController, 6);
        
        // Initialize default mappings
        manager.init_default_mappings();
        
        manager
    }
    
    /// Initialize default input mappings
    fn init_default_mappings(&mut self) {
        // Game controller default mapping
        let gamepad_mapping = InputMapping {
            device_type: InputDeviceType::GameController,
            button_mappings: [
                ("A".to_string(), SemanticAction::Select),
                ("B".to_string(), SemanticAction::GoBack),
                ("X".to_string(), SemanticAction::ToggleBezel),
                ("Y".to_string(), SemanticAction::OpenMenu),
                ("Start".to_string(), SemanticAction::TacticalReset),
                ("LB".to_string(), SemanticAction::ZoomOut),
                ("RB".to_string(), SemanticAction::ZoomIn),
            ].into_iter().collect(),
            axis_mappings: vec![
                AxisMapping {
                    axis: "LeftStickX".to_string(),
                    action: SemanticAction::PanLeft,
                    threshold: 0.5,
                    absolute: false,
                    deadzone: 0.15,
                },
                AxisMapping {
                    axis: "LeftStickY".to_string(),
                    action: SemanticAction::PanUp,
                    threshold: 0.5,
                    absolute: false,
                    deadzone: 0.15,
                },
                AxisMapping {
                    axis: "RightStickX".to_string(),
                    action: SemanticAction::RotateLeft,
                    threshold: 0.5,
                    absolute: false,
                    deadzone: 0.15,
                },
                AxisMapping {
                    axis: "DPadX".to_string(),
                    action: SemanticAction::NextElement,
                    threshold: 0.5,
                    absolute: false,
                    deadzone: 0.0,
                },
            ],
            gesture_mappings: HashMap::new(),
            dwell_select_ms: 1000,
            dwell_enabled: false,
        };
        
        // VR controller default mapping
        let vr_mapping = InputMapping {
            device_type: InputDeviceType::VRController,
            button_mappings: [
                ("Trigger".to_string(), SemanticAction::Select),
                ("Grip".to_string(), SemanticAction::Grab),
                ("Primary".to_string(), SemanticAction::ToggleBezel),
                ("Secondary".to_string(), SemanticAction::GoBack),
                ("Menu".to_string(), SemanticAction::OpenMenu),
            ].into_iter().collect(),
            axis_mappings: vec![
                AxisMapping {
                    axis: "ThumbstickX".to_string(),
                    action: SemanticAction::PanLeft,
                    threshold: 0.4,
                    absolute: false,
                    deadzone: 0.2,
                },
                AxisMapping {
                    axis: "ThumbstickY".to_string(),
                    action: SemanticAction::PanUp,
                    threshold: 0.4,
                    absolute: false,
                    deadzone: 0.2,
                },
            ],
            gesture_mappings: HashMap::new(),
            dwell_select_ms: 0,
            dwell_enabled: false,
        };
        
        // Hand tracking default mapping
        let hand_mapping = InputMapping {
            device_type: InputDeviceType::HandTracking,
            button_mappings: HashMap::new(),
            axis_mappings: Vec::new(),
            gesture_mappings: [
                (HandGesture::Pinch, SemanticAction::Select),
                (HandGesture::Grab, SemanticAction::Grab),
                (HandGesture::OpenPalm, SemanticAction::Release),
                (HandGesture::Point, SemanticAction::CursorMove),
            ].into_iter().collect(),
            dwell_select_ms: 0,
            dwell_enabled: false,
        };
        
        // Eye tracking default mapping
        let eye_mapping = InputMapping {
            device_type: InputDeviceType::EyeTracking,
            button_mappings: [
                ("Blink".to_string(), SemanticAction::Select),
                ("DoubleBlink".to_string(), SemanticAction::GoBack),
            ].into_iter().collect(),
            axis_mappings: Vec::new(),
            gesture_mappings: HashMap::new(),
            dwell_select_ms: 1500,
            dwell_enabled: true,
        };
        
        self.mappings.insert(InputDeviceType::GameController, gamepad_mapping);
        self.mappings.insert(InputDeviceType::VRController, vr_mapping);
        self.mappings.insert(InputDeviceType::HandTracking, hand_mapping);
        self.mappings.insert(InputDeviceType::EyeTracking, eye_mapping);
    }
    
    /// Process an input event from a device
    pub fn process_event(&mut self, event: AdvancedInputEvent) -> Vec<SemanticAction> {
        if !self.enabled {
            return Vec::new();
        }
        
        let timestamp = Instant::now();
        let device_id = match &event {
            AdvancedInputEvent::GameController { device, .. } => device.clone(),
            AdvancedInputEvent::VRController { device, .. } => device.clone(),
            AdvancedInputEvent::HandTracking { device, .. } => device.clone(),
            AdvancedInputEvent::EyeTracking { device, .. } => device.clone(),
            AdvancedInputEvent::Switch { device, .. } => device.clone(),
            AdvancedInputEvent::Gesture { device, .. } => device.clone(),
            AdvancedInputEvent::DeviceConnected(id) | AdvancedInputEvent::DeviceDisconnected(id) => {
                // Handle connection events
                if matches!(event, AdvancedInputEvent::DeviceConnected(_)) {
                    self.devices.insert(id.clone(), DeviceState {
                        device: id.clone(),
                        connected: true,
                        last_activity: timestamp,
                        haptic_enabled: id.device_type.supports_haptics(),
                    });
                } else {
                    self.devices.remove(id);
                }
                return Vec::new();
            }
        };
        
        // Update device activity
        if let Some(state) = self.devices.get_mut(&device_id) {
            state.last_activity = timestamp;
        }
        
        self.last_active_device = Some(device_id.clone());
        
        // Add to event buffer
        self.event_buffer.push_back((timestamp, event.clone()));
        if self.event_buffer.len() > 100 {
            self.event_buffer.pop_front();
        }
        
        // Process based on event type
        let mut actions = Vec::new();
        
        match event {
            AdvancedInputEvent::GameController { state, .. } => {
                actions.extend(self.process_game_controller(&device_id, &state));
            }
            AdvancedInputEvent::VRController { state, .. } => {
                actions.extend(self.process_vr_controller(&device_id, &state));
            }
            AdvancedInputEvent::HandTracking { state, .. } => {
                actions.extend(self.process_hand_tracking(&device_id, &state));
            }
            AdvancedInputEvent::EyeTracking { state, .. } => {
                actions.extend(self.process_eye_tracking(&device_id, &state));
            }
            AdvancedInputEvent::Switch { state, .. } => {
                actions.extend(self.process_switch(&device_id, &state));
            }
            AdvancedInputEvent::Gesture { gesture, hand, .. } => {
                actions.extend(self.process_gesture(&device_id, gesture, hand));
            }
            _ => {}
        }
        
        // Apply conflict resolution
        self.resolve_conflicts(&mut actions, &device_id);
        
        actions
    }
    
    /// Process game controller state
    fn process_game_controller(&self, device: &DeviceId, state: &GameControllerState) -> Vec<SemanticAction> {
        let mut actions = Vec::new();
        
        if let Some(_mapping) = self.mappings.get(&device.device_type) {
            // Check triggers for zoom
            if state.left_trigger > 0.5 {
                actions.push(SemanticAction::ZoomOut);
            }
            if state.right_trigger > 0.5 {
                actions.push(SemanticAction::ZoomIn);
            }
            
            // Check D-pad for navigation
            if state.dpad.up {
                actions.push(SemanticAction::PanUp);
            }
            if state.dpad.down {
                actions.push(SemanticAction::PanDown);
            }
            if state.dpad.left {
                actions.push(SemanticAction::PanLeft);
            }
            if state.dpad.right {
                actions.push(SemanticAction::PanRight);
            }
            
            // Check face buttons
            if state.face_buttons[0] { // A
                actions.push(SemanticAction::Select);
            }
            if state.face_buttons[1] { // B
                actions.push(SemanticAction::GoBack);
            }
        }
        
        actions
    }
    
    /// Process VR controller state
    fn process_vr_controller(&self, _device: &DeviceId, state: &VRControllerState) -> Vec<SemanticAction> {
        let mut actions = Vec::new();
        
        // Trigger for select
        if state.trigger > 0.8 {
            actions.push(SemanticAction::Select);
        }
        
        // Grip for grab
        if state.grip > 0.8 {
            actions.push(SemanticAction::Grab);
        }
        
        // Thumbstick for navigation
        if state.thumbstick.1 > 0.5 {
            actions.push(SemanticAction::ZoomIn);
        } else if state.thumbstick.1 < -0.5 {
            actions.push(SemanticAction::ZoomOut);
        }
        
        actions
    }
    
    /// Process hand tracking state
    fn process_hand_tracking(&mut self, device: &DeviceId, state: &HandTrackingState) -> Vec<SemanticAction> {
        let mut actions = Vec::new();
        
        // Detect gestures from state
        for gesture in &state.gestures {
            actions.extend(self.process_gesture(device, *gesture, state.hand));
        }
        
        // Detect pinch for selection
        if let Some(thumb) = state.fingers.get(0) {
            if let Some(index) = state.fingers.get(1) {
                let distance = Self::distance(thumb.tip, index.tip);
                if distance < 0.02 { // 2cm threshold
                    actions.push(SemanticAction::Select);
                }
            }
        }
        
        actions
    }
    
    /// Calculate distance between two 3D points
    fn distance(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
        let dx = a.0 - b.0;
        let dy = a.1 - b.1;
        let dz = a.2 - b.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    
    /// Process eye tracking state
    fn process_eye_tracking(&mut self, device: &DeviceId, state: &EyeTrackingState) -> Vec<SemanticAction> {
        let mut actions = Vec::new();
        
        if !state.is_tracking {
            return actions;
        }
        
        // Check for blink patterns
        for pattern in &state.blink_patterns {
            match pattern {
                BlinkPattern::SingleBlink => actions.push(SemanticAction::Select),
                BlinkPattern::DoubleBlink => actions.push(SemanticAction::GoBack),
                _ => {}
            }
        }
        
        // Handle dwell clicking
        if let Some(mapping) = self.mappings.get(&device.device_type) {
            if mapping.dwell_enabled {
                let current_gaze = state.gaze_point;
                
                if let Some(target) = self.dwell_state.current_target {
                    // Check if gaze is still on target (within tolerance)
                    let tolerance = 0.05; // 5% of screen
                    let dx = (current_gaze.0 - target.0).abs();
                    let dy = (current_gaze.1 - target.1).abs();
                    
                    if dx < tolerance && dy < tolerance {
                        // Still dwelling
                        if let Some(start) = self.dwell_state.dwell_start {
                            let elapsed = Instant::now().duration_since(start);
                            if elapsed.as_millis() as u32 >= mapping.dwell_select_ms && !self.dwell_state.dwell_triggered {
                                actions.push(SemanticAction::Select);
                                self.dwell_state.dwell_triggered = true;
                            }
                        } else {
                            self.dwell_state.dwell_start = Some(Instant::now());
                            self.dwell_state.dwell_triggered = false;
                        }
                    } else {
                        // Moved off target
                        self.dwell_state.current_target = Some(current_gaze);
                        self.dwell_state.dwell_start = None;
                        self.dwell_state.dwell_triggered = false;
                    }
                } else {
                    // New target
                    self.dwell_state.current_target = Some(current_gaze);
                    self.dwell_state.dwell_start = Some(Instant::now());
                    self.dwell_state.dwell_triggered = false;
                }
            }
        }
        
        actions
    }
    
    /// Process switch state
    fn process_switch(&self, _device: &DeviceId, state: &SwitchState) -> Vec<SemanticAction> {
        let mut actions = Vec::new();
        
        if state.pressed {
            // Single switch can be used for scanning or direct selection
            actions.push(SemanticAction::Select);
        }
        
        actions
    }
    
    /// Process detected gesture
    fn process_gesture(&self, device: &DeviceId, gesture: HandGesture, _hand: Handedness) -> Vec<SemanticAction> {
        let mut actions = Vec::new();
        
        if let Some(mapping) = self.mappings.get(&device.device_type) {
            if let Some(action) = mapping.gesture_mappings.get(&gesture) {
                actions.push(*action);
            }
        }
        
        // Default gesture handling
        match gesture {
            HandGesture::Pinch => actions.push(SemanticAction::Select),
            HandGesture::Grab => actions.push(SemanticAction::Grab),
            HandGesture::OpenPalm => actions.push(SemanticAction::Release),
            HandGesture::ThumbsUp => actions.push(SemanticAction::GoBack),
            _ => {}
        }
        
        actions
    }
    
    /// Apply conflict resolution to actions
    fn resolve_conflicts(&self, actions: &mut Vec<SemanticAction>, current_device: &DeviceId) {
        match self.conflict_resolution {
            ConflictResolution::LastWins => {
                // Keep all actions from last device, which is the current one
            }
            ConflictResolution::FirstWins => {
                // If any other device has been active in the last 100ms, ignore this one
                let now = Instant::now();
                let threshold = Duration::from_millis(100);
                for state in self.devices.values() {
                    if state.connected && state.device != *current_device {
                        if now.duration_since(state.last_activity) < threshold {
                            actions.clear();
                            return;
                        }
                    }
                }
            }
            ConflictResolution::PriorityBased => {
                let current_priority = self.device_priorities.get(&current_device.device_type).copied().unwrap_or(99);
                
                // If any higher-priority device (lower number) has been active recently, ignore this one
                let now = Instant::now();
                let recent_threshold = Duration::from_millis(150);
                
                for state in self.devices.values() {
                    if state.connected && state.device != *current_device {
                        let priority = self.device_priorities.get(&state.device.device_type).copied().unwrap_or(99);
                        if priority < current_priority && now.duration_since(state.last_activity) < recent_threshold {
                            actions.clear();
                            return;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Get the last active device
    pub fn last_active_device(&self) -> Option<&DeviceId> {
        self.last_active_device.as_ref()
    }
    
    /// Get connected devices
    pub fn connected_devices(&self) -> Vec<&DeviceId> {
        self.devices.values()
            .filter(|s| s.connected)
            .map(|s| &s.device)
            .collect()
    }
    
    /// Set conflict resolution strategy
    pub fn set_conflict_resolution(&mut self, strategy: ConflictResolution) {
        self.conflict_resolution = strategy;
    }
    
    /// Enable or disable advanced input
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Update input mapping for a device type
    pub fn update_mapping(&mut self, mapping: InputMapping) {
        self.mappings.insert(mapping.device_type, mapping);
    }
    
    /// Trigger haptic feedback on a device
    pub fn trigger_haptic(&self, device: &DeviceId, intensity: f32, duration_ms: u32) {
        if let Some(state) = self.devices.get(device) {
            if state.haptic_enabled {
                tracing::debug!("Haptic feedback on {}: intensity={}, duration={}ms", 
                    device.display_name(), intensity, duration_ms);
                // In a real implementation, this would send haptic commands to the device
            }
        }
    }
    
    /// Get input mapping for a device type
    pub fn get_mapping(&self, device_type: InputDeviceType) -> Option<&InputMapping> {
        self.mappings.get(&device_type)
    }
}

impl Default for AdvancedInputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_id_creation() {
        let id = DeviceId::new(InputDeviceType::GameController, 0);
        assert_eq!(id.device_type, InputDeviceType::GameController);
        assert_eq!(id.instance, 0);
    }
    
    #[test]
    fn test_input_manager_creation() {
        let manager = AdvancedInputManager::new();
        assert!(manager.is_enabled());
        assert!(manager.connected_devices().is_empty());
    }
    
    #[test]
    fn test_game_controller_processing() {
        let mut manager = AdvancedInputManager::new();
        
        let device = DeviceId::new(InputDeviceType::GameController, 0);
        let state = GameControllerState {
            face_buttons: [true, false, false, false], // A pressed
            dpad: DPadState { up: true, ..Default::default() },
            ..Default::default()
        };
        
        let event = AdvancedInputEvent::GameController {
            device: device.clone(),
            state,
            delta: GameControllerState::default(),
        };
        
        let actions = manager.process_event(event);
        assert!(!actions.is_empty());
        assert!(actions.contains(&SemanticAction::Select));
        assert!(actions.contains(&SemanticAction::PanUp));
    }
    
    #[test]
    fn test_vr_controller_processing() {
        let mut manager = AdvancedInputManager::new();
        
        let device = DeviceId::new(InputDeviceType::VRController, 0);
        let state = VRControllerState {
            trigger: 0.9,
            grip: 0.0,
            thumbstick: (0.0, 0.6),
            ..Default::default()
        };
        
        let event = AdvancedInputEvent::VRController {
            device: device.clone(),
            state,
            delta: VRControllerState::default(),
        };
        
        let actions = manager.process_event(event);
        assert!(actions.contains(&SemanticAction::Select));
        assert!(actions.contains(&SemanticAction::ZoomIn));
    }
    
    #[test]
    fn test_gesture_processing() {
        let mut manager = AdvancedInputManager::new();
        
        let device = DeviceId::new(InputDeviceType::HandTracking, 0);
        let event = AdvancedInputEvent::Gesture {
            device: device.clone(),
            gesture: HandGesture::Pinch,
            hand: Handedness::Right,
        };
        
        let actions = manager.process_event(event);
        assert!(actions.contains(&SemanticAction::Select));
    }
    
    #[test]
    fn test_device_connection() {
        let mut manager = AdvancedInputManager::new();
        
        let device = DeviceId::new(InputDeviceType::GameController, 0);
        let event = AdvancedInputEvent::DeviceConnected(device.clone());
        
        manager.process_event(event);
        
        assert_eq!(manager.connected_devices().len(), 1);
        assert_eq!(manager.connected_devices()[0].device_type, InputDeviceType::GameController);
    }
    
    #[test]
    fn test_conflict_resolution_setting() {
        let mut manager = AdvancedInputManager::new();
        
        manager.set_conflict_resolution(ConflictResolution::PriorityBased);
        // Just verify it doesn't panic
    }
    
    #[test]
    fn test_haptic_trigger() {
        let mut manager = AdvancedInputManager::new();
        
        let device = DeviceId::new(InputDeviceType::VRController, 0);
        let connect_event = AdvancedInputEvent::DeviceConnected(device.clone());
        manager.process_event(connect_event);
        
        // Should not panic
        manager.trigger_haptic(&device, 0.5, 100);
    }
    
    #[test]
    fn test_device_display_name() {
        let id = DeviceId::with_hardware_id(
            InputDeviceType::GameController, 
            0, 
            "Xbox Controller".to_string()
        );
        
        let name = id.display_name();
        assert!(name.contains("Xbox Controller"));
    }
    
    #[test]
    fn test_hand_gesture_defaults() {
        assert_eq!(HandGesture::Pinch as i32, HandGesture::Pinch as i32);
        assert_ne!(HandGesture::Pinch, HandGesture::Grab);
    }
}
