//! Auditory Interface - Earcons System
//!
//! Provides audio feedback for navigation, commands, system status,
//! collaboration events, and UI interactions.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Categories of earcons for different system events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarconCategory {
    /// Navigation sounds (zoom, level changes, focus changes)
    Navigation,
    /// Command feedback (accepted, error, dangerous command warning)
    CommandFeedback,
    /// System status (notifications, alerts, battery)
    SystemStatus,
    /// Collaboration events (user join/leave, cursor sharing)
    Collaboration,
    /// Bezel and UI interactions (expand/collapse, button hover, mode switch)
    BezelUi,
}

impl EarconCategory {
    /// Get the default volume for this category (0.0 - 1.0)
    pub fn default_volume(&self) -> f32 {
        match self {
            Self::Navigation => 0.7,
            Self::CommandFeedback => 0.8,
            Self::SystemStatus => 0.9,
            Self::Collaboration => 0.6,
            Self::BezelUi => 0.5,
        }
    }
    
    /// Get description of the category
    pub fn description(&self) -> &'static str {
        match self {
            Self::Navigation => "Navigation sounds",
            Self::CommandFeedback => "Command feedback",
            Self::SystemStatus => "System status alerts",
            Self::Collaboration => "Collaboration events",
            Self::BezelUi => "UI interactions",
        }
    }
}

/// Specific earcon events within categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarconEvent {
    // Navigation
    ZoomIn,
    ZoomOut,
    LevelChange,
    FocusChange,
    SplitViewCreated,
    SplitViewClosed,
    
    // Command Feedback
    CommandAccepted,
    CommandError,
    CommandCompleted,
    DangerousCommandWarning,
    AutoCompleteSuggestion,
    
    // System Status
    Notification,
    TacticalAlert,
    BatteryLow,
    BatteryCritical,
    PerformanceWarning,
    
    // Collaboration
    UserJoined,
    UserLeft,
    CursorShared,
    FollowingStarted,
    FollowingEnded,
    
    // Bezel/UI
    BezelExpand,
    BezelCollapse,
    ButtonHover,
    ModeSwitch,
    ToggleHiddenFiles,
}

impl EarconEvent {
    /// Get the category for this event
    pub fn category(&self) -> EarconCategory {
        match self {
            // Navigation
            Self::ZoomIn | Self::ZoomOut | Self::LevelChange | Self::FocusChange |
            Self::SplitViewCreated | Self::SplitViewClosed => EarconCategory::Navigation,
            
            // Command Feedback
            Self::CommandAccepted | Self::CommandError | Self::CommandCompleted |
            Self::DangerousCommandWarning | Self::AutoCompleteSuggestion => EarconCategory::CommandFeedback,
            
            // System Status
            Self::Notification | Self::TacticalAlert | Self::BatteryLow |
            Self::BatteryCritical | Self::PerformanceWarning => EarconCategory::SystemStatus,
            
            // Collaboration
            Self::UserJoined | Self::UserLeft | Self::CursorShared |
            Self::FollowingStarted | Self::FollowingEnded => EarconCategory::Collaboration,
            
            // Bezel/UI
            Self::BezelExpand | Self::BezelCollapse | Self::ButtonHover |
            Self::ModeSwitch | Self::ToggleHiddenFiles => EarconCategory::BezelUi,
        }
    }
    
    /// Get the default priority (higher = more important, less likely to be skipped)
    pub fn priority(&self) -> u8 {
        match self {
            // Critical - never skip
            Self::DangerousCommandWarning | Self::TacticalAlert | Self::BatteryCritical => 10,
            
            // High priority
            Self::CommandError | Self::PerformanceWarning | Self::BatteryLow |
            Self::UserJoined | Self::UserLeft => 8,
            
            // Medium priority
            Self::CommandAccepted | Self::CommandCompleted | Self::Notification |
            Self::ZoomIn | Self::ZoomOut | Self::LevelChange => 5,
            
            // Low priority - can be skipped if too many sounds
            Self::ButtonHover | Self::AutoCompleteSuggestion | Self::FocusChange => 2,
            
            // Very low priority
            Self::BezelExpand | Self::BezelCollapse | Self::ModeSwitch |
            Self::ToggleHiddenFiles | Self::CursorShared | Self::FollowingStarted |
            Self::FollowingEnded | Self::SplitViewCreated | Self::SplitViewClosed => 1,
        }
    }
    
    /// Get a description of the sound pattern (for documentation and theming)
    pub fn sound_pattern(&self) -> &'static str {
        match self {
            // Navigation - ascending/descending tones
            Self::ZoomIn => "ascending_chime",
            Self::ZoomOut => "descending_chime",
            Self::LevelChange => "whoosh_transition",
            Self::FocusChange => "soft_tick",
            Self::SplitViewCreated => "split_chime",
            Self::SplitViewClosed => "merge_chime",
            
            // Command Feedback - distinct confirmation sounds
            Self::CommandAccepted => "positive_beep",
            Self::CommandError => "error_buzz",
            Self::CommandCompleted => "success_chime",
            Self::DangerousCommandWarning => "warning_alert",
            Self::AutoCompleteSuggestion => "soft_pop",
            
            // System Status - attention-getting sounds
            Self::Notification => "notification_ping",
            Self::TacticalAlert => "urgent_alarm",
            Self::BatteryLow => "battery_warning",
            Self::BatteryCritical => "critical_alarm",
            Self::PerformanceWarning => "performance_chirp",
            
            // Collaboration - social sounds
            Self::UserJoined => "door_chime",
            Self::UserLeft => "exit_whoosh",
            Self::CursorShared => "share_ping",
            Self::FollowingStarted => "follow_beep",
            Self::FollowingEnded => "unfollow_beep",
            
            // Bezel/UI - subtle feedback
            Self::BezelExpand => "expand_slide",
            Self::BezelCollapse => "collapse_slide",
            Self::ButtonHover => "hover_tick",
            Self::ModeSwitch => "mode_switch",
            Self::ToggleHiddenFiles => "toggle_click",
        }
    }
}

/// Spatial audio position for 3D sound positioning
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SpatialPosition {
    /// X position (-1.0 = left, 0.0 = center, 1.0 = right)
    pub x: f32,
    /// Y position (-1.0 = down, 0.0 = center, 1.0 = up)
    pub y: f32,
    /// Z position (distance, 0.0 = near, 1.0 = far)
    pub z: f32,
}

impl SpatialPosition {
    /// Create a centered position
    pub fn center() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
    
    /// Create a position from sector relative coordinates
    pub fn from_sector_position(sector_x: f32, sector_y: f32, distance: f32) -> Self {
        Self {
            x: sector_x.clamp(-1.0, 1.0),
            y: sector_y.clamp(-1.0, 1.0),
            z: distance.clamp(0.0, 1.0),
        }
    }
    
    /// Get left/right pan value (-1.0 to 1.0)
    pub fn pan(&self) -> f32 {
        self.x
    }
    
    /// Get volume attenuation based on distance (0.0 to 1.0)
    pub fn attenuation(&self) -> f32 {
        1.0 - (self.z * 0.5) // Reduce volume by up to 50% for distant sounds
    }
}

/// Configuration for a specific earcon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarconConfig {
    /// The event this config applies to
    pub event: EarconEvent,
    /// Volume multiplier (0.0 - 1.0)
    pub volume: f32,
    /// Whether spatial audio is enabled for this earcon
    pub spatial: bool,
    /// Playback speed (1.0 = normal)
    pub speed: f32,
    /// Whether to allow overlapping playback
    pub allow_overlap: bool,
    /// Minimum time between repeats
    pub debounce_duration: Duration,
}

impl Default for EarconConfig {
    fn default() -> Self {
        Self {
            event: EarconEvent::CommandAccepted,
            volume: 1.0,
            spatial: false,
            speed: 1.0,
            allow_overlap: true,
            debounce_duration: Duration::from_millis(50),
        }
    }
}

/// Earcon player manages audio feedback
pub struct EarconPlayer {
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
    /// Per-category volumes
    category_volumes: HashMap<EarconCategory, f32>,
    /// Per-event configurations
    event_configs: HashMap<EarconEvent, EarconConfig>,
    /// Last playback time for each event (for debouncing)
    last_played: HashMap<EarconEvent, Instant>,
    /// Active sounds for overlap management
    active_sounds_count: usize,
    /// Maximum concurrent sounds
    max_concurrent: usize,
    /// Whether spatial audio is enabled
    enabled: bool,
    /// Whether spatial audio is enabled
    spatial_audio_enabled: bool,
    
    /// Kira manager
    #[cfg(feature = "accessibility")]
    manager: Option<kira::manager::AudioManager<kira::manager::backend::cpal::CpalBackend>>,
    /// UI track bus
    #[cfg(feature = "accessibility")]
    ui_track: Option<kira::track::TrackHandle>,
}

impl std::fmt::Debug for EarconPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EarconPlayer")
            .field("master_volume", &self.master_volume)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl EarconPlayer {
    /// Create a new earcon player with default settings
    pub fn new() -> Self {
        #[cfg(feature = "accessibility")]
        let mut manager = match kira::manager::AudioManager::<kira::manager::backend::cpal::CpalBackend>::new(kira::manager::AudioManagerSettings::default()) {
            Ok(m) => Some(m),
            Err(_) => None,
        };

        #[cfg(feature = "accessibility")]
        let mut ui_track = None;
        #[cfg(feature = "accessibility")]
        if let Some(ref mut m) = manager {
            ui_track = m.add_sub_track(kira::track::TrackBuilder::default()).ok();
        }

        let mut player = Self {
            master_volume: 1.0,
            category_volumes: HashMap::new(),
            event_configs: HashMap::new(),
            last_played: HashMap::new(),
            active_sounds_count: 0,
            max_concurrent: 8,
            enabled: true,
            spatial_audio_enabled: true,
            #[cfg(feature = "accessibility")]
            manager,
            #[cfg(feature = "accessibility")]
            ui_track,
        };
        
        // Initialize default category volumes
        for category in [
            EarconCategory::Navigation,
            EarconCategory::CommandFeedback,
            EarconCategory::SystemStatus,
            EarconCategory::Collaboration,
            EarconCategory::BezelUi,
        ] {
            player.category_volumes.insert(category, category.default_volume());
        }
        
        player
    }
    
    /// Play an earcon for the given event
    pub fn play(&mut self, event: EarconEvent) {
        self.play_spatial(event, SpatialPosition::center());
    }
    
    /// Play an earcon with spatial positioning
    pub fn play_spatial(&mut self, event: EarconEvent, position: SpatialPosition) {
        if !self.enabled {
            return;
        }
        
        let config = self.get_config(event);
        
        if let Some(last) = self.last_played.get(&event) {
            if Instant::now().duration_since(*last) < config.debounce_duration {
                return;
            }
        }
        
        if self.active_sounds_count >= self.max_concurrent {
            if event.priority() < 5 { // Only skip low/medium priority when busy
                return;
            }
        }
        
        let volume = self.calculate_volume(event, &position);
        self.last_played.insert(event, Instant::now());
        
        #[cfg(feature = "accessibility")]
        if let (Some(ref mut manager), Some(ref track)) = (&mut self.manager, &self.ui_track) {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let sound_path = format!("{}/.local/share/tos/audio/{:?}.wav", home, event).to_lowercase();
            
            if std::path::Path::new(&sound_path).exists() {
                if let Ok(data) = kira::sound::static_sound::StaticSoundData::from_file(&sound_path) {
                    let mut settings = kira::sound::static_sound::StaticSoundSettings::new()
                        .output_destination(track)
                        .volume(volume as f64);
                    
                    settings.fade_in_tween = Some(kira::tween::Tween::default());
                    let _ = manager.play(data.with_settings(settings));
                }
            } else {
                // Tactical Fallback: Minimal blip using a short silence or just logging in the absence of assets
                tracing::debug!("Tactical Audio Asset Missing: {:?}", event);
            }
        }

        tracing::debug!(
            "Playing earcon: {:?} (category: {:?}, volume: {:.2}, pattern: {})",
            event,
            event.category(),
            volume,
            event.sound_pattern()
        );
    }
    
    fn calculate_volume(&self, event: EarconEvent, position: &SpatialPosition) -> f32 {
        let config = self.get_config_immutable(event);
        let category = event.category();
        let category_volume = self.category_volumes.get(&category).copied().unwrap_or(1.0);
        let mut volume = self.master_volume * category_volume * config.volume;
        if self.spatial_audio_enabled && config.spatial {
            volume *= position.attenuation();
        }
        volume.clamp(0.0, 1.0)
    }
    
    fn get_config(&mut self, event: EarconEvent) -> EarconConfig {
        self.event_configs.get(&event).cloned().unwrap_or_else(|| {
            let mut config = EarconConfig::default();
            config.event = event;
            self.event_configs.insert(event, config.clone());
            config
        })
    }
    
    fn get_config_immutable(&self, event: EarconEvent) -> EarconConfig {
        self.event_configs.get(&event).cloned().unwrap_or_else(|| {
            let mut config = EarconConfig::default();
            config.event = event;
            config
        })
    }
    
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }
    
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }
    
    pub fn set_category_volume(&mut self, category: EarconCategory, volume: f32) {
        self.category_volumes.insert(category, volume.clamp(0.0, 1.0));
    }
    
    pub fn category_volume(&self, category: EarconCategory) -> f32 {
        self.category_volumes.get(&category).copied().unwrap_or(1.0)
    }
    
    pub fn configure_event(&mut self, config: EarconConfig) {
        self.event_configs.insert(config.event, config);
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn set_spatial_audio_enabled(&mut self, enabled: bool) {
        self.spatial_audio_enabled = enabled;
    }
    
    pub fn is_spatial_audio_enabled(&self) -> bool {
        self.spatial_audio_enabled
    }
    
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }
    
    pub fn active_sound_count(&self) -> usize {
        self.active_sounds_count
    }
    
    pub fn mute(&mut self) {
        self.enabled = false;
    }
    
    pub fn unmute(&mut self) {
        self.enabled = true;
    }
    
    pub fn zoom_in(&mut self) { self.play(EarconEvent::ZoomIn); }
    pub fn zoom_out(&mut self) { self.play(EarconEvent::ZoomOut); }
    pub fn command_accepted(&mut self) { self.play(EarconEvent::CommandAccepted); }
    pub fn command_error(&mut self) { self.play(EarconEvent::CommandError); }
    pub fn dangerous_command_warning(&mut self) { self.play(EarconEvent::DangerousCommandWarning); }
    pub fn notification(&mut self) { self.play(EarconEvent::Notification); }
    pub fn tactical_alert(&mut self) { self.play(EarconEvent::TacticalAlert); }
    pub fn user_joined(&mut self, position: Option<SpatialPosition>) { self.play_spatial(EarconEvent::UserJoined, position.unwrap_or_default()); }
    pub fn user_left(&mut self, position: Option<SpatialPosition>) { self.play_spatial(EarconEvent::UserLeft, position.unwrap_or_default()); }
    pub fn bezel_expand(&mut self) { self.play(EarconEvent::BezelExpand); }
    pub fn bezel_collapse(&mut self) { self.play(EarconEvent::BezelCollapse); }
    
    pub fn reset_to_defaults(&mut self) {
        self.master_volume = 1.0;
        self.category_volumes.clear();
        for category in [
            EarconCategory::Navigation,
            EarconCategory::CommandFeedback,
            EarconCategory::SystemStatus,
            EarconCategory::Collaboration,
            EarconCategory::BezelUi,
        ] {
            self.category_volumes.insert(category, category.default_volume());
        }
        self.event_configs.clear();
        self.enabled = true;
        self.spatial_audio_enabled = false;
    }
}

impl Default for EarconPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_earcon_player_creation() {
        let player = EarconPlayer::new();
        assert!(player.is_enabled());
        assert_eq!(player.master_volume(), 1.0);
    }
}
