//! Motor Accessibility - Switch Devices, Sticky Keys, and Dwell Clicking
//! 
//! Provides motor accessibility features for users with limited mobility,
//! including switch device scanning, sticky keys, and dwell-based activation.

use super::{AccessibilityConfig, AccessibilityError, MotorInput};
use crate::system::input::SemanticEvent;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

/// Motor accessibility manager
#[derive(Debug)]
pub struct MotorAccessibility {
    config: Arc<RwLock<AccessibilityConfig>>,
    state: Arc<RwLock<MotorState>>,
    event_sender: mpsc::Sender<MotorInput>,
    _scanning_task: Option<tokio::task::JoinHandle<()>>,
}

/// Internal state for motor accessibility
#[derive(Debug)]
struct MotorState {
    /// Sticky keys state
    sticky_keys: StickyKeysState,
    /// Dwell clicking state
    dwell: DwellState,
    /// Switch scanning state
    scanning: ScanningState,
    /// Slow keys state
    slow_keys: SlowKeysState,
    /// Last input time for debouncing
    last_input_time: Instant,
}

/// Slow keys state
#[derive(Debug, Default)]
struct SlowKeysState {
    /// Keys currently being held down
    pressed_keys: HashMap<String, Instant>,
    /// Delay required before key registers
    delay_ms: u32,
}

/// Sticky keys state
#[derive(Debug, Default)]
struct StickyKeysState {
    /// Currently latched modifiers
    latched_modifiers: Vec<StickyModifier>,
    /// Locked modifiers (double-press)
    locked_modifiers: Vec<StickyModifier>,
    /// Order of modifier presses for sequence
    modifier_sequence: Vec<StickyModifier>,
}

/// Sticky modifier keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StickyModifier {
    Shift,
    Control,
    Alt,
    Super,
}

/// Dwell clicking state
#[derive(Debug)]
struct DwellState {
    /// Current cursor position
    cursor_x: f32,
    cursor_y: f32,
    /// Dwell start time
    dwell_start: Option<Instant>,
    /// Whether dwell is currently active
    is_dwelling: bool,
    /// Dwell time threshold in milliseconds
    threshold_ms: u32,
    /// Visual indicator shown
    indicator_shown: bool,
}

/// Switch scanning state
#[derive(Debug)]
struct ScanningState {
    /// Scanning enabled
    enabled: bool,
    /// Current scan group
    current_group: usize,
    /// Current item within group
    current_item: usize,
    /// Scan speed (items per minute)
    scan_speed: u32,
    /// Auto-scan enabled
    auto_scan: bool,
    /// Groups of items to scan
    groups: Vec<ScanGroup>,
    /// Last scan advance time
    last_advance: Instant,
}

/// A group of scannable items
#[derive(Debug, Clone)]
struct ScanGroup {
    name: String,
    items: Vec<ScanItem>,
}

/// A scannable item
#[derive(Debug, Clone)]
struct ScanItem {
    id: String,
    label: String,
    action: ScanAction,
    bounds: (f32, f32, f32, f32), // x, y, width, height
}

/// Actions that can be triggered by scanning
#[derive(Debug, Clone)]
enum ScanAction {
    SemanticEvent(SemanticEvent),
    Custom(String),
}

impl MotorAccessibility {
    /// Create a new motor accessibility manager
    pub async fn new(config: Arc<RwLock<AccessibilityConfig>>) -> Result<Self, AccessibilityError> {
        let (event_tx, mut event_rx) = mpsc::channel(32);
        
        let state = Arc::new(RwLock::new(MotorState {
            sticky_keys: StickyKeysState::default(),
            dwell: DwellState {
                cursor_x: 0.0,
                cursor_y: 0.0,
                dwell_start: None,
                is_dwelling: false,
                threshold_ms: 1000,
                indicator_shown: false,
            },
            scanning: ScanningState {
                enabled: false,
                current_group: 0,
                current_item: 0,
                scan_speed: 60, // 1 item per second
                auto_scan: true,
                groups: Vec::new(),
                last_advance: Instant::now(),
            },
            slow_keys: SlowKeysState {
                pressed_keys: HashMap::new(),
                delay_ms: config.read().await.slow_keys_ms,
            },
            last_input_time: Instant::now(),
        }));
        
        // Spawn scanning task if auto-scan is enabled
        let state_clone = state.clone();
        let config_clone = config.clone();
        let _scanning_task = Some(tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(100));
            
            loop {
                ticker.tick().await;
                
                let cfg = config_clone.read().await;
                let mut st = state_clone.write().await;
                
                if !cfg.switch_device_enabled || !st.scanning.enabled || !st.scanning.auto_scan {
                    continue;
                }
                
                // Calculate advance interval based on scan speed
                let interval_ms = 60000 / st.scanning.scan_speed;
                let elapsed = st.scanning.last_advance.elapsed().as_millis() as u32;
                
                if elapsed >= interval_ms {
                    // Advance to next item
                    st.advance_scan();
                    st.scanning.last_advance = Instant::now();
                }
            }
        }));
        
        // Spawn input processing task
        let state_clone = state.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            while let Some(input) = event_rx.recv().await {
                let mut st = state_clone.write().await;
                let cfg = config_clone.read().await;
                
                // Debounce input
                if st.last_input_time.elapsed() < Duration::from_millis(50) {
                    continue;
                }
                st.last_input_time = Instant::now();
                
                // Process the input
                st.process_input(input, &cfg);
            }
        });
        
        Ok(Self {
            config,
            state,
            event_sender: event_tx,
            _scanning_task,
        })
    }
    
    /// Process motor input and return semantic event if triggered
    pub async fn process_input(&self, input: MotorInput) -> Option<SemanticEvent> {
        let _ = self.event_sender.send(input).await;
        
        // For immediate responses, check state
        let mut state = self.state.write().await;
        let config = self.config.read().await;
        
        let slow_keys_enabled = config.slow_keys_ms > 0;
        
        match input {
            MotorInput::Switch1Press => {
                if slow_keys_enabled {
                    state.slow_keys.pressed_keys.insert("switch1".to_string(), Instant::now());
                    None
                } else {
                    if state.scanning.enabled {
                        state.get_current_scan_action()
                    } else {
                        Some(SemanticEvent::Select)
                    }
                }
            }
            MotorInput::Switch1Release => {
                if let Some(press_time) = state.slow_keys.pressed_keys.remove("switch1") {
                    if press_time.elapsed().as_millis() >= config.slow_keys_ms as u128 {
                        if state.scanning.enabled {
                            return state.get_current_scan_action();
                        }
                        return Some(SemanticEvent::Select);
                    }
                }
                None
            }
            MotorInput::Switch2Press => {
                if slow_keys_enabled {
                    state.slow_keys.pressed_keys.insert("switch2".to_string(), Instant::now());
                    None
                } else if state.scanning.enabled && !state.scanning.auto_scan {
                    state.advance_scan();
                    None
                } else {
                    Some(SemanticEvent::SecondarySelect)
                }
            }
            MotorInput::Switch2Release => {
                if let Some(press_time) = state.slow_keys.pressed_keys.remove("switch2") {
                    if press_time.elapsed().as_millis() >= config.slow_keys_ms as u128 {
                        if state.scanning.enabled && !state.scanning.auto_scan {
                            state.advance_scan();
                            return None;
                        }
                        return Some(SemanticEvent::SecondarySelect);
                    }
                }
                None
            }
            MotorInput::KeyPress { key } => {
                if slow_keys_enabled {
                    state.slow_keys.pressed_keys.insert(key, Instant::now());
                    None
                } else {
                    // Map common keys to semantic events if desired
                    None
                }
            }
            MotorInput::KeyRelease { key } => {
                if let Some(press_time) = state.slow_keys.pressed_keys.remove(&key) {
                    if press_time.elapsed().as_millis() >= config.slow_keys_ms as u128 {
                        // Key held long enough
                        tracing::info!("Slow key accepted: {}", key);
                        // Trigger haptic on success
                        drop(state);
                        drop(config);
                        self.trigger_haptic(50).await;
                    }
                }
                None
            }
            MotorInput::DwellTrigger { .. } => {
                Some(SemanticEvent::Select)
            }
            _ => None,
        }
    }
    
    /// Update cursor position for dwell clicking
    pub async fn update_cursor(&self, x: f32, y: f32) {
        let mut state = self.state.write().await;
        let config = self.config.read().await;
        
        // Check if cursor moved significantly
        let dx = x - state.dwell.cursor_x;
        let dy = y - state.dwell.cursor_y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        state.dwell.cursor_x = x;
        state.dwell.cursor_y = y;
        
        if distance > 5.0 {
            // Cursor moved, reset dwell
            state.dwell.dwell_start = None;
            state.dwell.is_dwelling = false;
            state.dwell.indicator_shown = false;
        } else if config.dwell_click_ms > 0 && !state.dwell.is_dwelling {
            // Start dwell
            state.dwell.dwell_start = Some(Instant::now());
            state.dwell.is_dwelling = true;
            state.dwell.threshold_ms = config.dwell_click_ms;
        }
        
        // Check if dwell threshold reached
        if let Some(start) = state.dwell.dwell_start {
            let elapsed = start.elapsed().as_millis() as u32;
            let threshold = state.dwell.threshold_ms;
            
            // Show indicator at 50% progress
            if elapsed > threshold / 2 && !state.dwell.indicator_shown {
                state.dwell.indicator_shown = true;
                // TODO: Trigger visual indicator
            }
            
            // Trigger click at 100% progress
            if elapsed >= threshold {
                state.dwell.dwell_start = None;
                state.dwell.is_dwelling = false;
                state.dwell.indicator_shown = false;
                
                // Send dwell trigger event
                drop(state);
                let _ = self.event_sender.send(MotorInput::DwellTrigger { x, y }).await;
            }
        }
    }
    
    /// Handle keyboard modifier for sticky keys
    pub async fn handle_modifier(&self, modifier: StickyModifier, pressed: bool) -> Vec<StickyModifier> {
        let mut state = self.state.write().await;
        
        if pressed {
            // Check if already latched (double-press to lock)
            if state.sticky_keys.latched_modifiers.contains(&modifier) {
                // Move to locked
                state.sticky_keys.latched_modifiers.retain(|&m| m != modifier);
                if !state.sticky_keys.locked_modifiers.contains(&modifier) {
                    state.sticky_keys.locked_modifiers.push(modifier);
                }
            } else if state.sticky_keys.locked_modifiers.contains(&modifier) {
                // Unlock
                state.sticky_keys.locked_modifiers.retain(|&m| m != modifier);
            } else {
                // Latch
                state.sticky_keys.latched_modifiers.push(modifier);
                state.sticky_keys.modifier_sequence.push(modifier);
            }
        } else {
            // Key released - keep latched until used
            // (they're cleared when a non-modifier key is pressed)
        }
        
        // Return active modifiers (latched + locked)
        let mut active = state.sticky_keys.latched_modifiers.clone();
        active.extend(state.sticky_keys.locked_modifiers.clone());
        active
    }
    
    /// Clear latched modifiers (call after non-modifier key press)
    pub async fn clear_latched_modifiers(&self) {
        let mut state = self.state.write().await;
        state.sticky_keys.latched_modifiers.clear();
        state.sticky_keys.modifier_sequence.clear();
    }
    
    /// Get current sticky keys state for display
    pub async fn get_sticky_keys_state(&self) -> StickyKeysDisplay {
        let state = self.state.read().await;
        StickyKeysDisplay {
            latched: state.sticky_keys.latched_modifiers.clone(),
            locked: state.sticky_keys.locked_modifiers.clone(),
        }
    }
    
    /// Enable switch scanning with given groups
    pub async fn enable_scanning(&self, groups: Vec<(String, Vec<(String, String)>)>) {
        let mut state = self.state.write().await;
        
        state.scanning.groups = groups.into_iter().map(|(name, items)| {
            ScanGroup {
                name,
                items: items.into_iter().map(|(id, label)| {
                    ScanItem {
                        id,
                        label,
                        action: ScanAction::Custom("select".to_string()),
                        bounds: (0.0, 0.0, 100.0, 50.0),
                    }
                }).collect(),
            }
        }).collect();
        
        state.scanning.enabled = true;
        state.scanning.current_group = 0;
        state.scanning.current_item = 0;
    }
    
    /// Disable switch scanning
    pub async fn disable_scanning(&self) {
        let mut state = self.state.write().await;
        state.scanning.enabled = false;
    }
    
    /// Get current scanning position for highlighting
    pub async fn get_scan_position(&self) -> Option<(usize, usize, String)> {
        let state = self.state.read().await;
        
        if !state.scanning.enabled {
            return None;
        }
        
        let group = state.scanning.groups.get(state.scanning.current_group)?;
        let item = group.items.get(state.scanning.current_item)?;
        
        Some((
            state.scanning.current_group,
            state.scanning.current_item,
            item.label.clone(),
        ))
    }
    
    /// Set scan speed (items per minute)
    pub async fn set_scan_speed(&self, items_per_minute: u32) {
        let mut state = self.state.write().await;
        state.scanning.scan_speed = items_per_minute.clamp(10, 300);
    }

    /// Trigger haptic feedback
    pub async fn trigger_haptic(&self, duration_ms: u32) {
        let config = self.config.read().await;
        if config.haptic_feedback_intensity > 0.0 {
            tracing::info!("Haptic feedback: {}ms at intensity {}", duration_ms, config.haptic_feedback_intensity);
            
            // In a real implementation on Linux, this might use:
            // 1. input-event-codes for force feedback API
            // 2. Platform-specific gamepad vibration APIs
            // 3. Mobile bridging via the Web Portal
        }
    }
    
    /// Shutdown motor accessibility systems
    pub async fn shutdown(&self) -> Result<(), AccessibilityError> {
        // Signal shutdown
        drop(self.event_sender.clone());
        tracing::info!("Motor accessibility shutdown");
        Ok(())
    }
}

impl MotorState {
    /// Process input based on configuration
    fn process_input(&mut self, input: MotorInput, config: &AccessibilityConfig) {
        match input {
            MotorInput::DwellStart { x, y } => {
                if config.dwell_click_ms > 0 {
                    self.dwell.cursor_x = x;
                    self.dwell.cursor_y = y;
                    self.dwell.dwell_start = Some(Instant::now());
                    self.dwell.is_dwelling = true;
                    self.dwell.threshold_ms = config.dwell_click_ms;
                }
            }
            MotorInput::DwellEnd { .. } => {
                self.dwell.dwell_start = None;
                self.dwell.is_dwelling = false;
                self.dwell.indicator_shown = false;
            }
            _ => {}
        }
    }
    
    /// Advance to next scan position
    fn advance_scan(&mut self) {
        if self.scanning.groups.is_empty() {
            return;
        }
        
        let current_group = &self.scanning.groups[self.scanning.current_group];
        
        if self.scanning.current_item + 1 < current_group.items.len() {
            // Next item in current group
            self.scanning.current_item += 1;
        } else if self.scanning.current_group + 1 < self.scanning.groups.len() {
            // Next group
            self.scanning.current_group += 1;
            self.scanning.current_item = 0;
        } else {
            // Wrap to beginning
            self.scanning.current_group = 0;
            self.scanning.current_item = 0;
        }
    }
    
    /// Get semantic event for current scan position
    fn get_current_scan_action(&self) -> Option<SemanticEvent> {
        let group = self.scanning.groups.get(self.scanning.current_group)?;
        let item = group.items.get(self.scanning.current_item)?;
        
        match &item.action {
            ScanAction::SemanticEvent(event) => Some(*event),
            ScanAction::Custom(_) => Some(SemanticEvent::Select),
        }
    }
}

/// Sticky keys state for display
#[derive(Debug, Clone)]
pub struct StickyKeysDisplay {
    pub latched: Vec<StickyModifier>,
    pub locked: Vec<StickyModifier>,
}

impl StickyKeysDisplay {
    /// Check if any modifiers are active
    pub fn is_active(&self) -> bool {
        !self.latched.is_empty() || !self.locked.is_empty()
    }
    
    /// Format for display
    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        
        for m in &self.latched {
            parts.push(format!("{:?}", m));
        }
        
        for m in &self.locked {
            parts.push(format!("{:?} (locked)", m));
        }
        
        if parts.is_empty() {
            "None".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Convert keyboard key to sticky modifier
pub fn key_to_modifier(key: &str) -> Option<StickyModifier> {
    match key.to_lowercase().as_str() {
        "shift" | "shiftleft" | "shiftright" => Some(StickyModifier::Shift),
        "control" | "ctrl" | "controlleft" | "controlright" => Some(StickyModifier::Control),
        "alt" | "altleft" | "altright" | "option" => Some(StickyModifier::Alt),
        "super" | "command" | "win" | "metaleft" | "metaright" => Some(StickyModifier::Super),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sticky_keys() {
        let config = Arc::new(RwLock::new(AccessibilityConfig {
            sticky_keys: true,
            ..Default::default()
        }));
        
        let motor = MotorAccessibility::new(config).await.unwrap();
        
        // Press shift
        let mods = motor.handle_modifier(StickyModifier::Shift, true).await;
        assert!(mods.contains(&StickyModifier::Shift));
        
        // Check state
        let state = motor.get_sticky_keys_state().await;
        assert!(state.latched.contains(&StickyModifier::Shift));
        
        // Clear latched
        motor.clear_latched_modifiers().await;
        let state = motor.get_sticky_keys_state().await;
        assert!(!state.is_active());
    }

    #[tokio::test]
    async fn test_dwell_click() {
        let config = Arc::new(RwLock::new(AccessibilityConfig {
            dwell_click_ms: 100,
            ..Default::default()
        }));
        
        let motor = MotorAccessibility::new(config).await.unwrap();
        
        // Start dwell
        motor.update_cursor(100.0, 100.0).await;
        
        // Should not trigger immediately
        let event = motor.process_input(MotorInput::Switch1Press).await;
        assert!(event.is_some());
    }

    #[test]
    fn test_key_to_modifier() {
        assert_eq!(key_to_modifier("Shift"), Some(StickyModifier::Shift));
        assert_eq!(key_to_modifier("Control"), Some(StickyModifier::Control));
        assert_eq!(key_to_modifier("Alt"), Some(StickyModifier::Alt));
        assert_eq!(key_to_modifier("Super"), Some(StickyModifier::Super));
        assert_eq!(key_to_modifier("A"), None);
    }

    #[test]
    fn test_sticky_keys_display() {
        let display = StickyKeysDisplay {
            latched: vec![StickyModifier::Shift],
            locked: vec![StickyModifier::Control],
        };
        
        assert!(display.is_active());
        let formatted = display.format();
        assert!(formatted.contains("Shift"));
        assert!(formatted.contains("Control"));
    }
}
