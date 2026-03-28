//! Android Input — touch and gesture tracking for the Android Face.



/// Android Input Source — handles touch and gesture recognition.
#[derive(Default)]
pub struct AndroidInput {
    /// Current touch position
    pub touch_pos: Option<(f32, f32)>,
    /// Active gesture state
    pub gesture_state: GestureState,
    /// Haptic feedback controller
    pub haptic: Option<HapticFeedbackController>,
}

/// Tracks a multi-touch gesture from start to current position.
#[derive(Default, Clone)]
pub struct GestureState {
    pub gesture_type: GestureType,
    pub start_x: Option<f32>,
    pub start_y: Option<f32>,
    pub current_x: Option<f32>,
    pub current_y: Option<f32>,
    pub distance: Option<f32>,
}

/// Recognized gesture types.
#[derive(Default, Clone, Debug)]
pub enum GestureType {
    #[default]
    None,
    PinchIn,
    PinchOut,
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    Tap,
    LongPress,
}

impl AndroidInput {
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin tracking a gesture from the initial touch point.
    pub fn init_gesture(&mut self, x: f32, y: f32) {
        self.gesture_state = GestureState {
            gesture_type: GestureType::None,
            start_x: Some(x),
            start_y: Some(y),
            current_x: Some(x),
            current_y: Some(y),
            distance: None,
        };
    }

    /// Update the gesture with a new touch position.
    pub fn update_gesture(&mut self, x: f32, y: f32) {
        if let (Some(start_x), Some(start_y)) =
            (self.gesture_state.start_x, self.gesture_state.start_y)
        {
            let dx = x - start_x;
            let dy = y - start_y;
            self.gesture_state.distance = Some((dx * dx + dy * dy).sqrt());
            self.gesture_state.current_x = Some(x);
            self.gesture_state.current_y = Some(y);
        }
    }

    /// Read the current detected gesture (if any).
    pub fn detect_gesture(&self) -> Option<GestureType> {
        match &self.gesture_state.gesture_type {
            GestureType::None => None,
            other => Some(other.clone()),
        }
    }

    /// Reset gesture tracking state.
    pub fn reset(&mut self) {
        self.gesture_state = GestureState::default();
    }

    /// Trigger haptic feedback with the given vibration pattern (ms values).
    pub fn vibrate(&self, pattern: &[u32]) {
        if let Some(ref haptic) = self.haptic {
            haptic.vibrate(pattern);
        }
    }
}

/// Haptic feedback controller — wraps the Android Vibrator service.
#[derive(Default)]
pub struct HapticFeedbackController {
    pub pattern: Option<Vec<u32>>,
}

impl HapticFeedbackController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Vibrate with a custom pattern.
    pub fn vibrate(&self, pattern: &[u32]) {
        // In production: android.os.Vibrator.vibrate()
        tracing::debug!("Android Face: Haptic pattern: {:?}", pattern);
    }

    /// Short tactile click.
    pub fn vibrate_short(&self) {
        self.vibrate(&[0, 50]);
    }

    /// Long press confirmation.
    pub fn vibrate_long(&self) {
        self.vibrate(&[0, 500]);
    }
}
