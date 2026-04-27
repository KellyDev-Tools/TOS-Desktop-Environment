//! §14.4: Device Support & Mapping
//!
//! Provides a user-configurable mapping from game controller and spatial inputs
//! to `SemanticEvent` action identifiers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A mapping for a specific game controller or spatial input device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMapping {
    /// Mapping from controller buttons (e.g. "South", "East") to action IDs.
    pub button_mappings: HashMap<String, String>,
    /// Mapping from spatial gestures (e.g. "pinch_left", "wrist_tap") to action IDs.
    pub gesture_mappings: HashMap<String, String>,
    /// Threshold for axis movement to trigger an action.
    pub axis_threshold: f32,
    /// Dwell time in milliseconds for gaze to trigger a select event.
    pub gaze_dwell_ms: u32,
}

impl Default for DeviceMapping {
    fn default() -> Self {
        let mut button_mappings = HashMap::new();
        // Default controller mappings (common patterns)
        button_mappings.insert("South".to_string(), "select".to_string());
        button_mappings.insert("East".to_string(), "home".to_string());
        button_mappings.insert("North".to_string(), "zoom_out".to_string());
        button_mappings.insert("West".to_string(), "zoom_in".to_string());
        button_mappings.insert("Start".to_string(), "command_hub".to_string());
        button_mappings.insert("Select".to_string(), "toggle_bezel".to_string());

        let mut gesture_mappings = HashMap::new();
        // §14.4 OpenXR defaults
        gesture_mappings.insert("pinch_left".to_string(), "zoom_out".to_string());
        gesture_mappings.insert("pinch_right".to_string(), "zoom_in".to_string());
        gesture_mappings.insert("wrist_tap".to_string(), "open_hub".to_string());

        Self {
            button_mappings,
            gesture_mappings,
            axis_threshold: 0.5,
            gaze_dwell_ms: 500,
        }
    }
}

impl DeviceMapping {
    /// Look up the action for a controller button.
    pub fn lookup_button(&self, button: &str) -> Option<&str> {
        self.button_mappings.get(button).map(|s| s.as_str())
    }

    /// Look up the action for a spatial gesture.
    pub fn lookup_gesture(&self, gesture: &str) -> Option<&str> {
        self.gesture_mappings.get(gesture).map(|s| s.as_str())
    }
}
