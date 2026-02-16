//! TOS Accessibility Module - Phase 10 Implementation
//! 
//! Provides comprehensive accessibility support including:
//! - Screen reader integration (AT-SPI/Orca)
//! - Auditory interface (earcons, TTS)
//! - Visual accessibility (high contrast, themes)
//! - Motor accessibility (switch devices, sticky keys)

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod audio;
pub mod visual;
pub mod motor;
pub mod screen_reader;
pub mod cognitive;

pub use audio::AuditoryInterface;
pub use visual::VisualAccessibility;
pub use motor::MotorAccessibility;
pub use screen_reader::ScreenReader;
pub use cognitive::CognitiveAccessibility;

/// Accessibility configuration for TOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Enable screen reader integration
    pub screen_reader_enabled: bool,
    /// Enable auditory feedback (earcons)
    pub auditory_feedback: bool,
    /// Enable text-to-speech for notifications
    pub tts_enabled: bool,
    /// Visual theme mode
    pub theme_mode: ThemeMode,
    /// Font scale factor (1.0 = normal)
    pub font_scale: f32,
    /// Enable high contrast
    pub high_contrast: bool,
    /// Colorblind filter type
    pub colorblind_filter: Option<ColorblindFilter>,
    /// Enable switch device support
    pub switch_device_enabled: bool,
    /// Sticky keys enabled
    pub sticky_keys: bool,
    /// Dwell click time in milliseconds (0 = disabled)
    pub dwell_click_ms: u32,
    /// Slow keys delay in milliseconds (0 = disabled)
    pub slow_keys_ms: u32,
    /// Haptic feedback intensity (0.0 to 1.0)
    pub haptic_feedback_intensity: f32,
    /// Enable simplified mode (reduced clutter)
    pub simplified_mode: bool,
    /// Enable Braille display output
    pub braille_output_enabled: bool,
    /// Enable tutorial mode
    pub tutorial_mode: bool,
    /// Sound theme name
    pub sound_theme: String,
    /// TTS voice settings
    pub tts_voice: String,
    pub tts_rate: f32,
    pub tts_pitch: f32,
    /// Verbosity level for announcements
    pub verbosity: VerbosityLevel,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_enabled: false,
            auditory_feedback: true,
            tts_enabled: false,
            theme_mode: ThemeMode::System,
            font_scale: 1.0,
            high_contrast: false,
            colorblind_filter: None,
            switch_device_enabled: false,
            sticky_keys: false,
            dwell_click_ms: 0,
            slow_keys_ms: 0,
            haptic_feedback_intensity: 0.0,
            simplified_mode: false,
            braille_output_enabled: false,
            tutorial_mode: false,
            sound_theme: "default".to_string(),
            tts_voice: "default".to_string(),
            tts_rate: 1.0,
            tts_pitch: 1.0,
            verbosity: VerbosityLevel::Normal,
        }
    }
}

/// Visual theme modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
    HighContrast,
}

/// Colorblind filter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorblindFilter {
    Deuteranopia, // Red-green (green weak)
    Protanopia,   // Red-green (red weak)
    Tritanopia,   // Blue-yellow
    Achromatopsia, // Complete color blindness
}

/// Verbosity levels for screen reader/tts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbosityLevel {
    Minimal,  // Only critical information
    Normal,   // Standard navigation feedback
    Verbose,  // Detailed descriptions
    Debug,    // Everything including technical details
}

/// Accessibility announcement types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessibilityAnnouncement {
    /// Navigation event (zoom, level change)
    Navigation {
        from_level: String,
        to_level: String,
        description: String,
    },
    /// Action confirmation
    Action {
        action: String,
        result: String,
    },
    /// Status update
    Status {
        component: String,
        state: String,
    },
    /// Error/warning
    Alert {
        severity: AlertSeverity,
        message: String,
    },
    /// Collaboration event
    Collaboration {
        event_type: String,
        participant: String,
    },
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Main accessibility manager
#[derive(Debug)]
pub struct AccessibilityManager {
    config: Arc<RwLock<AccessibilityConfig>>,
    auditory: Option<AuditoryInterface>,
    visual: VisualAccessibility,
    motor: Option<MotorAccessibility>,
    screen_reader: Option<ScreenReader>,
    cognitive: CognitiveAccessibility,
}

impl AccessibilityManager {
    /// Create a new accessibility manager with the given configuration
    pub async fn new(config: AccessibilityConfig) -> Result<Self, AccessibilityError> {
        let config = Arc::new(RwLock::new(config));
        
        // Initialize auditory interface if enabled
        let auditory = if config.read().await.auditory_feedback || config.read().await.tts_enabled {
            Some(AuditoryInterface::new(config.clone()).await?)
        } else {
            None
        };
        
        // Initialize visual accessibility
        let visual = VisualAccessibility::new(config.clone()).await;
        
        // Initialize motor accessibility if enabled
        let motor = if config.read().await.switch_device_enabled {
            Some(MotorAccessibility::new(config.clone()).await?)
        } else {
            None
        };
        
        // Initialize screen reader if enabled
        let screen_reader = if config.read().await.screen_reader_enabled {
            Some(ScreenReader::new(config.clone()).await?)
        } else {
            None
        };
        
        // Initialize cognitive accessibility
        let cognitive = CognitiveAccessibility::new(config.clone()).await;
        
        Ok(Self {
            config,
            auditory,
            visual: visual,
            motor,
            screen_reader,
            cognitive,
        })
    }
    
    /// Announce an accessibility event
    pub async fn announce(&self, announcement: AccessibilityAnnouncement) {
        let config = self.config.read().await;
        
        // Send to screen reader if enabled
        if config.screen_reader_enabled {
            if let Some(ref sr) = self.screen_reader {
                let _ = sr.announce(&announcement).await;
            }
        }
        
        // Play earcon if auditory feedback enabled
        if config.auditory_feedback {
            if let Some(ref audio) = self.auditory {
                let _ = audio.play_announcement(&announcement).await;
            }
        }
        
        // Speak with TTS if enabled
        if config.tts_enabled {
            if let Some(ref audio) = self.auditory {
                let mut text = format_announcement(&announcement, config.verbosity);
                
                // Add cognitive guidance if in tutorial mode
                if config.tutorial_mode {
                    if let AccessibilityAnnouncement::Navigation { from_level, to_level, .. } = &announcement {
                        if let Some(guidance) = self.cognitive.get_guidance(from_level, to_level).await {
                            text = format!("{}. Guidance: {}", text, guidance);
                        }
                    }
                }
                
                let _ = audio.speak(&text).await;
            }
        }
        
        tracing::info!("Accessibility announcement: {:?}", announcement);
    }
    
    /// Get current accessibility configuration
    pub async fn get_config(&self) -> AccessibilityConfig {
        self.config.read().await.clone()
    }
    
    /// Update accessibility configuration
    pub async fn update_config(&self, new_config: AccessibilityConfig) -> Result<(), AccessibilityError> {
        let mut config = self.config.write().await;
        *config = new_config;
        // TODO: Reinitialize components if needed
        Ok(())
    }
    
    /// Get CSS classes for current accessibility settings
    pub async fn get_css_classes(&self) -> String {
        self.visual.get_css_classes().await
    }
    
    /// Process motor accessibility input
    pub async fn process_motor_input(&self, input: MotorInput) -> Option<crate::system::input::SemanticEvent> {
        if let Some(ref motor) = self.motor {
            motor.process_input(input).await
        } else {
            None
        }
    }
    
    /// Shutdown accessibility systems
    pub async fn shutdown(&self) {
        if let Some(ref sr) = self.screen_reader {
            let _ = sr.shutdown().await;
        }
        if let Some(ref audio) = self.auditory {
            let _ = audio.shutdown().await;
        }
        if let Some(ref motor) = self.motor {
            let _ = motor.shutdown().await;
        }
    }
}

/// Format an announcement based on verbosity level
fn format_announcement(announcement: &AccessibilityAnnouncement, verbosity: VerbosityLevel) -> String {
    match announcement {
        AccessibilityAnnouncement::Navigation { from_level, to_level, description } => {
            match verbosity {
                VerbosityLevel::Minimal => format!("{} to {}", from_level, to_level),
                VerbosityLevel::Normal => format!("Navigated to {}", to_level),
                VerbosityLevel::Verbose => format!("Navigated from {} to {}. {}", from_level, to_level, description),
                VerbosityLevel::Debug => format!("NAV: {} -> {} | {}", from_level, to_level, description),
            }
        }
        AccessibilityAnnouncement::Action { action, result } => {
            match verbosity {
                VerbosityLevel::Minimal => action.clone(),
                VerbosityLevel::Normal => format!("{}: {}", action, result),
                VerbosityLevel::Verbose => format!("Action {} completed with result: {}", action, result),
                VerbosityLevel::Debug => format!("ACTION[{}] = {}", action, result),
            }
        }
        AccessibilityAnnouncement::Status { component, state } => {
            match verbosity {
                VerbosityLevel::Minimal => state.clone(),
                VerbosityLevel::Normal => format!("{} is {}", component, state),
                VerbosityLevel::Verbose => format!("{} status changed to {}", component, state),
                VerbosityLevel::Debug => format!("STATUS[{}] = {}", component, state),
            }
        }
        AccessibilityAnnouncement::Alert { severity, message } => {
            let severity_str = format!("{:?}", severity);
            match verbosity {
                VerbosityLevel::Minimal => message.clone(),
                VerbosityLevel::Normal => format!("{}: {}", severity_str, message),
                VerbosityLevel::Verbose => format!("Alert - {}: {}", severity_str, message),
                VerbosityLevel::Debug => format!("ALERT[{:?}]: {}", severity, message),
            }
        }
        AccessibilityAnnouncement::Collaboration { event_type, participant } => {
            match verbosity {
                VerbosityLevel::Minimal => participant.clone(),
                VerbosityLevel::Normal => format!("{} {}", participant, event_type),
                VerbosityLevel::Verbose => format!("{} has {}", participant, event_type),
                VerbosityLevel::Debug => format!("COLLAB[{}]: {}", participant, event_type),
            }
        }
    }
}

/// Motor input types for switch devices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorInput {
    Switch1Press,
    Switch1Release,
    Switch2Press,
    Switch2Release,
    KeyPress { key: String },
    KeyRelease { key: String },
    DwellStart { x: f32, y: f32 },
    DwellEnd { x: f32, y: f32 },
    DwellTrigger { x: f32, y: f32 },
}

/// Accessibility error types
#[derive(Debug, thiserror::Error)]
pub enum AccessibilityError {
    #[error("Audio system error: {0}")]
    AudioError(String),
    #[error("Screen reader error: {0}")]
    ScreenReaderError(String),
    #[error("Motor input error: {0}")]
    MotorError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
}

/// Create default accessibility configuration
pub fn default_config() -> AccessibilityConfig {
    AccessibilityConfig::default()
}

/// Create high contrast configuration
pub fn high_contrast_config() -> AccessibilityConfig {
    AccessibilityConfig {
        high_contrast: true,
        theme_mode: ThemeMode::HighContrast,
        font_scale: 1.25,
        auditory_feedback: true,
        ..Default::default()
    }
}

/// Create screen reader optimized configuration
pub fn screen_reader_config() -> AccessibilityConfig {
    AccessibilityConfig {
        screen_reader_enabled: true,
        tts_enabled: true,
        verbosity: VerbosityLevel::Verbose,
        auditory_feedback: true,
        ..Default::default()
    }
}

/// Create motor accessibility configuration
pub fn motor_accessibility_config() -> AccessibilityConfig {
    AccessibilityConfig {
        switch_device_enabled: true,
        sticky_keys: true,
        dwell_click_ms: 1000,
        auditory_feedback: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert!(!config.screen_reader_enabled);
        assert!(config.auditory_feedback);
        assert_eq!(config.font_scale, 1.0);
    }

    #[test]
    fn test_high_contrast_config() {
        let config = high_contrast_config();
        assert!(config.high_contrast);
        assert_eq!(config.theme_mode, ThemeMode::HighContrast);
        assert!(config.font_scale > 1.0);
    }

    #[test]
    fn test_announcement_formatting() {
        let announcement = AccessibilityAnnouncement::Navigation {
            from_level: "Global".to_string(),
            to_level: "Command Hub".to_string(),
            description: "Sector Alpha".to_string(),
        };
        
        let minimal = format_announcement(&announcement, VerbosityLevel::Minimal);
        assert!(minimal.contains("Global"));
        assert!(minimal.contains("Command Hub"));
        
        let verbose = format_announcement(&announcement, VerbosityLevel::Verbose);
        assert!(verbose.contains("Sector Alpha"));
    }
}
