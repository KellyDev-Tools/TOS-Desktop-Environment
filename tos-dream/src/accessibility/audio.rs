//! Auditory Interface - Earcons and Text-to-Speech
//! 
//! Provides audio feedback for navigation, actions, and alerts.
//! Supports multiple sound themes and TTS integration.

use super::{AccessibilityAnnouncement, AccessibilityConfig, AccessibilityError, AlertSeverity};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Sound categories for earcons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Navigation,      // Zoom, level changes
    Selection,       // Select, click
    Command,         // Command execution
    System,          // Notifications, alerts
    Collaboration,   // User join/leave
    Bezel,           // UI interactions
}

/// Sound events within categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEvent {
    // Navigation
    ZoomIn,
    ZoomOut,
    LevelChange,
    FocusChange,
    
    // Selection
    Select,
    MultiSelect,
    Deselect,
    
    // Command
    CommandAccepted,
    CommandError,
    CommandComplete,
    DangerousCommand,
    
    // System
    Notification,
    AlertInfo,
    AlertWarning,
    AlertError,
    AlertCritical,
    Startup,
    Shutdown,
    
    // Collaboration
    UserJoin,
    UserLeave,
    CursorShare,
    
    // Bezel
    BezelExpand,
    BezelCollapse,
    ModeSwitch,
}

/// Audio interface for TOS accessibility
#[derive(Debug)]
pub struct AuditoryInterface {
    config: Arc<RwLock<AccessibilityConfig>>,
    sound_theme: Arc<RwLock<SoundTheme>>,
    tts_queue: mpsc::Sender<String>,
    #[cfg(feature = "accessibility")]
    _audio_thread: Option<std::thread::JoinHandle<()>>,
}

impl AuditoryInterface {
    /// Create a new auditory interface
    pub async fn new(config: Arc<RwLock<AccessibilityConfig>>) -> Result<Self, AccessibilityError> {
        let (tts_tx, mut tts_rx) = mpsc::channel(32);
        
        // Initialize sound theme
        let sound_theme = Arc::new(RwLock::new(SoundTheme::load_default().await));
        
        // Spawn audio processing thread
        #[cfg(feature = "accessibility")]
        let _audio_thread = {
            let theme = sound_theme.clone();
            let cfg = config.clone();
            Some(std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    while let Some(text) = tts_rx.recv().await {
                        let config = cfg.read().await;
                        if config.tts_enabled {
                            if let Err(e) = Self::speak_text(&text, &config).await {
                                tracing::warn!("TTS error: {}", e);
                            }
                        }
                    }
                });
            }))
        };
        
        #[cfg(not(feature = "accessibility"))]
        let _audio_thread = None;
        
        Ok(Self {
            config,
            sound_theme,
            tts_queue: tts_tx,
            _audio_thread,
        })
    }
    
    /// Play an earcon for a sound event
    pub async fn play(&self, event: SoundEvent) -> Result<(), AccessibilityError> {
        let config = self.config.read().await;
        if !config.auditory_feedback {
            return Ok(());
        }
        
        let theme = self.sound_theme.read().await;
        
        if let Some(sound) = theme.get_sound(event) {
            #[cfg(feature = "accessibility")]
            {
                self.play_sound(sound).await?;
            }
            
            tracing::debug!("Playing earcon: {:?}", event);
        }
        
        Ok(())
    }
    
    /// Play an earcon for an accessibility announcement
    pub async fn play_announcement(&self, announcement: &AccessibilityAnnouncement) -> Result<(), AccessibilityError> {
        let event = match announcement {
            AccessibilityAnnouncement::Navigation { .. } => SoundEvent::LevelChange,
            AccessibilityAnnouncement::Action { .. } => SoundEvent::CommandAccepted,
            AccessibilityAnnouncement::Status { .. } => SoundEvent::Notification,
            AccessibilityAnnouncement::Alert { severity, .. } => match severity {
                AlertSeverity::Info => SoundEvent::AlertInfo,
                AlertSeverity::Warning => SoundEvent::AlertWarning,
                AlertSeverity::Error => SoundEvent::AlertError,
                AlertSeverity::Critical => SoundEvent::AlertCritical,
            },
            AccessibilityAnnouncement::Collaboration { event_type, .. } => match event_type.as_str() {
                "joined" => SoundEvent::UserJoin,
                "left" => SoundEvent::UserLeave,
                _ => SoundEvent::Notification,
            },
        };
        
        self.play(event).await
    }
    
    /// Speak text using TTS
    pub async fn speak(&self, text: &str) -> Result<(), AccessibilityError> {
        let config = self.config.read().await;
        if !config.tts_enabled {
            return Ok(());
        }
        
        // Queue the text for speaking
        let _ = self.tts_queue.send(text.to_string()).await;
        
        tracing::debug!("TTS queued: {}", text);
        Ok(())
    }
    
    /// Speak text immediately (blocking)
    #[cfg(feature = "accessibility")]
    async fn speak_text(text: &str, config: &AccessibilityConfig) -> Result<(), AccessibilityError> {
        // Try speech-dispatcher on Linux
        #[cfg(target_os = "linux")]
        {
            match Self::speak_with_speech_dispatcher(text, config).await {
                Ok(_) => return Ok(()),
                Err(e) => tracing::warn!("Speech dispatcher failed: {}", e),
            }
        }
        
        // Fallback to console beep pattern for testing
        tracing::info!("TTS (simulated): {}", text);
        Ok(())
    }
    
    #[cfg(all(feature = "accessibility", target_os = "linux"))]
    async fn speak_with_speech_dispatcher(
        text: &str,
        config: &AccessibilityConfig,
    ) -> Result<(), AccessibilityError> {
        use std::process::Stdio;
        use tokio::process::Command;
        
        let mut cmd = Command::new("spd-say");
        cmd.arg("-t")
            .arg("text")
            .arg("-r")
            .arg(format!("{}", (config.tts_rate * 100.0) as i32))
            .arg("-p")
            .arg(format!("{}", (config.tts_pitch * 100.0) as i32))
            .arg(text)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        
        if !config.tts_voice.is_empty() && config.tts_voice != "default" {
            cmd.arg("-i").arg(&config.tts_voice);
        }
        
        let status = cmd.status().await.map_err(|e| {
            AccessibilityError::AudioError(format!("Failed to run spd-say: {}", e))
        })?;
        
        if !status.success() {
            return Err(AccessibilityError::AudioError(
                "spd-say returned non-zero exit code".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Play a sound using rodio
    #[cfg(feature = "accessibility")]
    async fn play_sound(&self, _sound: &Sound) -> Result<(), AccessibilityError> {
        // This would use rodio to play actual audio files
        // For now, just log that we would play the sound
        tracing::debug!("Would play sound: {:?}", _sound);
        Ok(())
    }
    
    /// Change sound theme
    pub async fn set_theme(&self, theme_name: &str) -> Result<(), AccessibilityError> {
        let mut theme = self.sound_theme.write().await;
        *theme = SoundTheme::load(theme_name).await?;
        Ok(())
    }
    
    /// Get current sound theme name
    pub async fn current_theme(&self) -> String {
        self.sound_theme.read().await.name.clone()
    }
    
    /// Shutdown the auditory interface
    pub async fn shutdown(&self) -> Result<(), AccessibilityError> {
        // Signal shutdown to audio thread
        drop(self.tts_queue.clone());
        tracing::info!("Auditory interface shutdown");
        Ok(())
    }
}

/// Sound definition
#[derive(Debug, Clone)]
pub struct Sound {
    pub name: String,
    pub category: SoundCategory,
    pub frequency: f32,      // For synthesized sounds
    pub duration_ms: u32,
    pub waveform: Waveform,
    pub volume: f32,         // 0.0 to 1.0
}

/// Waveform types for synthesized sounds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

/// Sound theme containing all earcons
#[derive(Debug)]
pub struct SoundTheme {
    pub name: String,
    sounds: HashMap<SoundEvent, Sound>,
}

impl SoundTheme {
    /// Load the default sound theme
    pub async fn load_default() -> Self {
        Self::load("default").await.unwrap_or_else(|_| Self::create_default_theme())
    }
    
    /// Load a named sound theme
    pub async fn load(name: &str) -> Result<Self, AccessibilityError> {
        // In a full implementation, this would load from files
        // For now, just create the default theme
        if name == "default" || name == "minimal" {
            Ok(Self::create_default_theme())
        } else if name == "sci-fi" {
            Ok(Self::create_scifi_theme())
        } else {
            Err(AccessibilityError::AudioError(format!(
                "Unknown sound theme: {}", name
            )))
        }
    }
    
    /// Get a sound for an event
    pub fn get_sound(&self, event: SoundEvent) -> Option<&Sound> {
        self.sounds.get(&event)
    }
    
    /// Create the default sound theme
    fn create_default_theme() -> Self {
        let mut sounds = HashMap::new();
        
        // Navigation sounds
        sounds.insert(SoundEvent::ZoomIn, Sound {
            name: "zoom_in".to_string(),
            category: SoundCategory::Navigation,
            frequency: 880.0,  // A5
            duration_ms: 100,
            waveform: Waveform::Sine,
            volume: 0.5,
        });
        
        sounds.insert(SoundEvent::ZoomOut, Sound {
            name: "zoom_out".to_string(),
            category: SoundCategory::Navigation,
            frequency: 440.0,  // A4
            duration_ms: 150,
            waveform: Waveform::Sine,
            volume: 0.5,
        });
        
        sounds.insert(SoundEvent::LevelChange, Sound {
            name: "level_change".to_string(),
            category: SoundCategory::Navigation,
            frequency: 660.0,
            duration_ms: 80,
            waveform: Waveform::Triangle,
            volume: 0.4,
        });
        
        // Selection sounds
        sounds.insert(SoundEvent::Select, Sound {
            name: "select".to_string(),
            category: SoundCategory::Selection,
            frequency: 1000.0,
            duration_ms: 50,
            waveform: Waveform::Sine,
            volume: 0.3,
        });
        
        // Command sounds
        sounds.insert(SoundEvent::CommandAccepted, Sound {
            name: "command_ok".to_string(),
            category: SoundCategory::Command,
            frequency: 784.0,  // G5
            duration_ms: 100,
            waveform: Waveform::Sine,
            volume: 0.4,
        });
        
        sounds.insert(SoundEvent::CommandError, Sound {
            name: "command_error".to_string(),
            category: SoundCategory::Command,
            frequency: 200.0,
            duration_ms: 200,
            waveform: Waveform::Sawtooth,
            volume: 0.6,
        });
        
        sounds.insert(SoundEvent::DangerousCommand, Sound {
            name: "dangerous".to_string(),
            category: SoundCategory::Command,
            frequency: 150.0,
            duration_ms: 300,
            waveform: Waveform::Square,
            volume: 0.7,
        });
        
        // Alert sounds
        sounds.insert(SoundEvent::AlertInfo, Sound {
            name: "alert_info".to_string(),
            category: SoundCategory::System,
            frequency: 500.0,
            duration_ms: 100,
            waveform: Waveform::Sine,
            volume: 0.4,
        });
        
        sounds.insert(SoundEvent::AlertWarning, Sound {
            name: "alert_warning".to_string(),
            category: SoundCategory::System,
            frequency: 300.0,
            duration_ms: 200,
            waveform: Waveform::Triangle,
            volume: 0.5,
        });
        
        sounds.insert(SoundEvent::AlertError, Sound {
            name: "alert_error".to_string(),
            category: SoundCategory::System,
            frequency: 150.0,
            duration_ms: 300,
            waveform: Waveform::Sawtooth,
            volume: 0.6,
        });
        
        sounds.insert(SoundEvent::AlertCritical, Sound {
            name: "alert_critical".to_string(),
            category: SoundCategory::System,
            frequency: 100.0,
            duration_ms: 500,
            waveform: Waveform::Square,
            volume: 0.8,
        });
        
        // Collaboration sounds
        sounds.insert(SoundEvent::UserJoin, Sound {
            name: "user_join".to_string(),
            category: SoundCategory::Collaboration,
            frequency: 600.0,
            duration_ms: 150,
            waveform: Waveform::Sine,
            volume: 0.4,
        });
        
        sounds.insert(SoundEvent::UserLeave, Sound {
            name: "user_leave".to_string(),
            category: SoundCategory::Collaboration,
            frequency: 400.0,
            duration_ms: 150,
            waveform: Waveform::Sine,
            volume: 0.4,
        });
        
        Self {
            name: "default".to_string(),
            sounds,
        }
    }
    
    /// Create a sci-fi themed sound set
    fn create_scifi_theme() -> Self {
        let mut theme = Self::create_default_theme();
        theme.name = "sci-fi".to_string();
        
        // Modify some sounds for sci-fi feel
        if let Some(sound) = theme.sounds.get_mut(&SoundEvent::ZoomIn) {
            sound.frequency = 1200.0;
            sound.waveform = Waveform::Square;
        }
        
        if let Some(sound) = theme.sounds.get_mut(&SoundEvent::ZoomOut) {
            sound.frequency = 600.0;
            sound.waveform = Waveform::Square;
        }
        
        theme
    }
}

/// Generate a beep pattern for testing without audio hardware
pub fn generate_beep_pattern(event: SoundEvent) -> String {
    match event {
        SoundEvent::ZoomIn => "♪".to_string(),
        SoundEvent::ZoomOut => "♫".to_string(),
        SoundEvent::Select => "•".to_string(),
        SoundEvent::CommandAccepted => "✓".to_string(),
        SoundEvent::CommandError => "✗".to_string(),
        SoundEvent::AlertWarning => "▲".to_string(),
        SoundEvent::AlertCritical => "⚠".to_string(),
        _ => "♪".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sound_theme_default() {
        let theme = SoundTheme::load_default().await;
        assert_eq!(theme.name, "default");
        assert!(theme.get_sound(SoundEvent::ZoomIn).is_some());
        assert!(theme.get_sound(SoundEvent::Select).is_some());
    }

    #[tokio::test]
    async fn test_sound_theme_scifi() {
        let theme = SoundTheme::load("sci-fi").await.unwrap();
        assert_eq!(theme.name, "sci-fi");
    }

    #[test]
    fn test_beep_pattern_generation() {
        assert_eq!(generate_beep_pattern(SoundEvent::Select), "•");
        assert_eq!(generate_beep_pattern(SoundEvent::CommandAccepted), "✓");
    }
}
