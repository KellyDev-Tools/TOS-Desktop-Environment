//! Auditory Interface - Earcons and Text-to-Speech
//! 
//! Provides audio feedback for navigation, actions, and alerts.
//! Supports multiple sound themes and TTS integration.

use super::{AccessibilityAnnouncement, AccessibilityConfig, AccessibilityError, AlertSeverity};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[cfg(feature = "accessibility")]
use kira::{
    manager::{AudioManager as KiraManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    tween::Tween,
};

// In test builds (or with --features test-audio) swap to Kira's zero-hardware MockBackend.
#[cfg(all(feature = "accessibility", any(test, feature = "test-audio")))]
type AccessKiraBackend = kira::manager::backend::mock::MockBackend;
#[cfg(all(feature = "accessibility", not(any(test, feature = "test-audio"))))]
type AccessKiraBackend = kira::manager::backend::cpal::CpalBackend;
use std::sync::Mutex;

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
pub struct AuditoryInterface {
    config: Arc<RwLock<AccessibilityConfig>>,
    sound_theme: Arc<RwLock<SoundTheme>>,
    tts_queue: mpsc::Sender<String>,
    #[cfg(feature = "accessibility")]
    manager: Option<Mutex<KiraManager<AccessKiraBackend>>>,
    #[cfg(feature = "accessibility")]
    _audio_thread: Option<std::thread::JoinHandle<()>>,
}

impl std::fmt::Debug for AuditoryInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditoryInterface")
            .field("config", &self.config)
            .finish()
    }
}

impl std::fmt::Debug for AuditoryInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditoryInterface")
            .field("config", &self.config)
            .finish()
    }
}

impl AuditoryInterface {
    /// Create a new auditory interface
    pub async fn new(config: Arc<RwLock<AccessibilityConfig>>) -> Result<Self, AccessibilityError> {
        let (tts_tx, mut tts_rx) = mpsc::channel::<String>(32);
        let (tts_tx, mut tts_rx) = mpsc::channel::<String>(32);
        
        // Initialize sound theme
        let sound_theme = Arc::new(RwLock::new(SoundTheme::load_default().await));
        
        #[cfg(feature = "accessibility")]
        let manager = match KiraManager::<AccessKiraBackend>::new(AudioManagerSettings::default()) {
            Ok(m) => Some(Mutex::new(m)),
            Err(e) => {
                tracing::error!("Failed to initialize Kira for Accessibility: {:?}", e);
                None
            }
        };

        // Spawn audio processing thread for TTS
        #[cfg(feature = "accessibility")]
        let _audio_thread = {
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
        #[cfg(not(feature = "accessibility"))]
        let manager = None;
        #[cfg(not(feature = "accessibility"))]
        let manager = None;
        
        Ok(Self {
            config,
            sound_theme,
            tts_queue: tts_tx,
            manager,
            manager,
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
                self.play_sound_by_name(&sound.name).await?;
                self.play_sound_by_name(&sound.name).await?;
            }
            
            tracing::debug!("Playing accessibility earcon: {:?}", event);
            tracing::debug!("Playing accessibility earcon: {:?}", event);
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
        #[cfg(target_os = "linux")]
        {
            match Self::speak_with_speech_dispatcher(text, config).await {
                Ok(_) => return Ok(()),
                Err(e) => tracing::warn!("Speech dispatcher failed: {}", e),
            }
        }
        
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
    
    /// Play a sound using Kira
    /// Play a sound using Kira
    #[cfg(feature = "accessibility")]
    async fn play_sound_by_name(&self, name: &str) -> Result<(), AccessibilityError> {
        if let Some(ref manager) = self.manager {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let sound_path = format!("{}/.local/share/tos/audio/{}.wav", home, name).to_lowercase();
            
            if std::path::Path::new(&sound_path).exists() {
                if let Ok(data) = StaticSoundData::from_file(&sound_path) {
                    if let Ok(mut mgr) = manager.lock() {
                        let _ = mgr.play(data.with_settings(StaticSoundSettings::new().fade_in_tween(Some(Tween::default()))));
                    }
                }
            }
        }
    async fn play_sound_by_name(&self, name: &str) -> Result<(), AccessibilityError> {
        if let Some(ref manager) = self.manager {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let sound_path = format!("{}/.local/share/tos/audio/{}.wav", home, name).to_lowercase();
            
            if std::path::Path::new(&sound_path).exists() {
                if let Ok(data) = StaticSoundData::from_file(&sound_path) {
                    if let Ok(mut mgr) = manager.lock() {
                        let _ = mgr.play(data.with_settings(StaticSoundSettings::new().fade_in_tween(Some(Tween::default()))));
                    }
                }
            }
        }
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
    pub frequency: f32,
    pub frequency: f32,
    pub duration_ms: u32,
    pub waveform: Waveform,
    pub volume: f32,
    pub volume: f32,
}

/// Waveform types
/// Waveform types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

/// Sound theme
/// Sound theme
#[derive(Debug)]
pub struct SoundTheme {
    pub name: String,
    sounds: HashMap<SoundEvent, Sound>,
}

impl SoundTheme {
    pub async fn load_default() -> Self {
        Self::load("default").await.unwrap_or_else(|_| Self::create_default_theme())
    }
    
    pub async fn load(name: &str) -> Result<Self, AccessibilityError> {
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
    
    pub fn get_sound(&self, event: SoundEvent) -> Option<&Sound> {
        self.sounds.get(&event)
    }
    
    fn create_default_theme() -> Self {
        let mut sounds = HashMap::new();
        
        sounds.insert(SoundEvent::ZoomIn, Sound {
            name: "zoom_in".to_string(),
            category: SoundCategory::Navigation,
            frequency: 880.0,
            frequency: 880.0,
            duration_ms: 100,
            waveform: Waveform::Sine,
            volume: 0.5,
        });
        
        sounds.insert(SoundEvent::ZoomOut, Sound {
            name: "zoom_out".to_string(),
            category: SoundCategory::Navigation,
            frequency: 440.0,
            frequency: 440.0,
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
        
        sounds.insert(SoundEvent::Select, Sound {
            name: "select".to_string(),
            category: SoundCategory::Selection,
            frequency: 1000.0,
            duration_ms: 50,
            waveform: Waveform::Sine,
            volume: 0.3,
        });
        
        sounds.insert(SoundEvent::CommandAccepted, Sound {
            name: "command_accepted".to_string(),
            name: "command_accepted".to_string(),
            category: SoundCategory::Command,
            frequency: 784.0,
            frequency: 784.0,
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
        
        sounds.insert(SoundEvent::Notification, Sound {
            name: "notification".to_string(),
        sounds.insert(SoundEvent::Notification, Sound {
            name: "notification".to_string(),
            category: SoundCategory::System,
            frequency: 500.0,
            duration_ms: 100,
            waveform: Waveform::Sine,
            volume: 0.4,
        });
        
        // Alert sounds for different severities
        sounds.insert(SoundEvent::AlertInfo, Sound {
            name: "alert_info".to_string(),
            category: SoundCategory::System,
            frequency: 600.0,
            duration_ms: 120,
            waveform: Waveform::Triangle,
            volume: 0.45,
        });

        sounds.insert(SoundEvent::AlertWarning, Sound {
            name: "alert_warning".to_string(),
            category: SoundCategory::System,
            frequency: 420.0,
            duration_ms: 160,
            waveform: Waveform::Sawtooth,
            volume: 0.55,
        });

        sounds.insert(SoundEvent::AlertError, Sound {
            name: "alert_error".to_string(),
            category: SoundCategory::System,
            frequency: 240.0,
            duration_ms: 220,
            waveform: Waveform::Square,
            volume: 0.7,
        });

        sounds.insert(SoundEvent::AlertCritical, Sound {
            name: "alert_critical".to_string(),
            category: SoundCategory::System,
            frequency: 160.0,
            duration_ms: 300,
            waveform: Waveform::Noise,
            volume: 0.9,
        });
        
        Self {
            name: "default".to_string(),
            sounds,
        }
    }
    
    fn create_scifi_theme() -> Self {
        let mut theme = Self::create_default_theme();
        theme.name = "sci-fi".to_string();
        theme
    }
}
