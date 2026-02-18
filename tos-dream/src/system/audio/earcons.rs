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
#[derive(Debug)]
pub struct EarconPlayer {
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
    /// Per-category volumes
    category_volumes: HashMap<EarconCategory, f32>,
    /// Per-event configurations
    event_configs: HashMap<EarconEvent, EarconConfig>,
    /// Last playback time for each event (for debouncing)
    last_played: HashMap<EarconEvent, Instant>,
    /// Currently playing sounds (for overlap management)
    active_sounds: Vec<EarconEvent>,
    /// Maximum concurrent sounds
    max_concurrent: usize,
    /// Whether spatial audio is enabled
    enabled: bool,
    /// Whether spatial audio is enabled
    spatial_audio_enabled: bool,
    /// Audio output stream handle
    #[cfg(feature = "accessibility")]
    #[serde(skip)]
    stream_handle: Option<rodio::OutputStreamHandle>,
    /// Background thread for audio (to keep stream alive)
    #[cfg(feature = "accessibility")]
    #[serde(skip)]
    _stream: Option<rodio::OutputStream>,
}

impl EarconPlayer {
    /// Create a new earcon player with default settings
    pub fn new() -> Self {
        #[cfg(feature = "accessibility")]
        let (stream, handle) = match rodio::OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };

        let mut player = Self {
            master_volume: 1.0,
            category_volumes: HashMap::new(),
            event_configs: HashMap::new(),
            last_played: HashMap::new(),
            active_sounds: Vec::new(),
            max_concurrent: 8,
            enabled: true,
            spatial_audio_enabled: true,
            #[cfg(feature = "accessibility")]
            stream_handle: handle,
            #[cfg(feature = "accessibility")]
            _stream: stream,
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
        
        // Check debounce - get config first to avoid borrow issues
        let config = self.get_config(event);
        
        if let Some(last) = self.last_played.get(&event) {
            if Instant::now().duration_since(*last) < config.debounce_duration {
                return;
            }
        }
        
        // Check if we should skip low-priority sounds when busy
        if self.active_sounds.len() >= self.max_concurrent {
            let priority = event.priority();
            let min_active_priority = self.active_sounds.iter()
                .map(|e| e.priority())
                .min()
                .unwrap_or(0);
            
            if priority <= min_active_priority {
                return; // Skip this sound
            }
        }
        
        // Calculate final volume
        let volume = self.calculate_volume(event, &position);
        
        // Record playback
        self.last_played.insert(event, Instant::now());
        
        // Actual audio playback via rodio implementation
        #[cfg(feature = "accessibility")]
        if let Some(handle) = &self.stream_handle {
            if let Ok(sink) = rodio::Sink::try_new(handle) {
                sink.set_volume(volume);
                
                match event {
                    EarconEvent::ZoomIn => {
                        sink.append(rodio::source::SineWave::new(880.0).take_duration(Duration::from_millis(50)));
                        sink.append(rodio::source::SineWave::new(1760.0).take_duration(Duration::from_millis(80)));
                    }
                    EarconEvent::ZoomOut => {
                        sink.append(rodio::source::SineWave::new(1760.0).take_duration(Duration::from_millis(50)));
                        sink.append(rodio::source::SineWave::new(880.0).take_duration(Duration::from_millis(80)));
                    }
                    EarconEvent::CommandAccepted | EarconEvent::CommandCompleted => {
                        sink.append(rodio::source::SineWave::new(440.0).take_duration(Duration::from_millis(100)));
                    }
                    EarconEvent::CommandError | EarconEvent::DangerousCommandWarning => {
                        sink.append(rodio::source::SineWave::new(110.0).take_duration(Duration::from_millis(200)));
                        sink.append(rodio::source::SineWave::new(110.0).take_duration(Duration::from_millis(200)));
                    }
                    EarconEvent::TacticalAlert => {
                        sink.append(rodio::source::SineWave::new(220.0).take_duration(Duration::from_millis(500)));
                    }
                    _ => {
                        // Default blip
                        sink.append(rodio::source::SineWave::new(660.0).take_duration(Duration::from_millis(30)));
                    }
                }
                sink.detach();
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
    
    /// Calculate the final volume for an event considering all factors
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
    
    /// Get configuration for an event (creating default if needed)
    fn get_config(&mut self, event: EarconEvent) -> EarconConfig {
        self.event_configs.get(&event).cloned().unwrap_or_else(|| {
            let mut config = EarconConfig::default();
            config.event = event;
            self.event_configs.insert(event, config.clone());
            config
        })
    }
    
    /// Get configuration for an event (immutable version, returns default if not found)
    fn get_config_immutable(&self, event: EarconEvent) -> EarconConfig {
        self.event_configs.get(&event).cloned().unwrap_or_else(|| {
            let mut config = EarconConfig::default();
            config.event = event;
            config
        })
    }
    
    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }
    
    /// Get master volume
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }
    
    /// Set volume for a category
    pub fn set_category_volume(&mut self, category: EarconCategory, volume: f32) {
        self.category_volumes.insert(category, volume.clamp(0.0, 1.0));
    }
    
    /// Get volume for a category
    pub fn category_volume(&self, category: EarconCategory) -> f32 {
        self.category_volumes.get(&category).copied().unwrap_or(1.0)
    }
    
    /// Configure a specific event
    pub fn configure_event(&mut self, config: EarconConfig) {
        self.event_configs.insert(config.event, config);
    }
    
    /// Enable or disable earcons globally
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if earcons are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Enable or disable spatial audio
    pub fn set_spatial_audio_enabled(&mut self, enabled: bool) {
        self.spatial_audio_enabled = enabled;
    }
    
    /// Check if spatial audio is enabled
    pub fn is_spatial_audio_enabled(&self) -> bool {
        self.spatial_audio_enabled
    }
    
    /// Set maximum concurrent sounds
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }
    
    /// Clear all active sounds (e.g., when switching contexts)
    pub fn clear_active_sounds(&mut self) {
        self.active_sounds.clear();
    }
    
    /// Get active sound count
    pub fn active_sound_count(&self) -> usize {
        self.active_sounds.len()
    }
    
    /// Mute all sounds temporarily
    pub fn mute(&mut self) {
        self.enabled = false;
    }
    
    /// Unmute sounds
    pub fn unmute(&mut self) {
        self.enabled = true;
    }
    
    /// Play navigation earcon for zoom in
    pub fn zoom_in(&mut self) {
        self.play(EarconEvent::ZoomIn);
    }
    
    /// Play navigation earcon for zoom out
    pub fn zoom_out(&mut self) {
        self.play(EarconEvent::ZoomOut);
    }
    
    /// Play command accepted earcon
    pub fn command_accepted(&mut self) {
        self.play(EarconEvent::CommandAccepted);
    }
    
    /// Play command error earcon
    pub fn command_error(&mut self) {
        self.play(EarconEvent::CommandError);
    }
    
    /// Play dangerous command warning
    pub fn dangerous_command_warning(&mut self) {
        self.play(EarconEvent::DangerousCommandWarning);
    }
    
    /// Play notification earcon
    pub fn notification(&mut self) {
        self.play(EarconEvent::Notification);
    }
    
    /// Play tactical alert earcon
    pub fn tactical_alert(&mut self) {
        self.play(EarconEvent::TacticalAlert);
    }
    
    /// Play user joined earcon with optional spatial position
    pub fn user_joined(&mut self, position: Option<SpatialPosition>) {
        let pos = position.unwrap_or_default();
        self.play_spatial(EarconEvent::UserJoined, pos);
    }
    
    /// Play user left earcon with optional spatial position
    pub fn user_left(&mut self, position: Option<SpatialPosition>) {
        let pos = position.unwrap_or_default();
        self.play_spatial(EarconEvent::UserLeft, pos);
    }
    
    /// Play bezel expand/collapse earcons
    pub fn bezel_expand(&mut self) {
        self.play(EarconEvent::BezelExpand);
    }
    
    pub fn bezel_collapse(&mut self) {
        self.play(EarconEvent::BezelCollapse);
    }
    
    /// Get all category volumes as a map
    pub fn all_category_volumes(&self) -> &HashMap<EarconCategory, f32> {
        &self.category_volumes
    }
    
    /// Reset all settings to defaults
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
    
    #[test]
    fn test_volume_settings() {
        let mut player = EarconPlayer::new();
        
        player.set_master_volume(0.5);
        assert_eq!(player.master_volume(), 0.5);
        
        player.set_category_volume(EarconCategory::Navigation, 0.3);
        assert_eq!(player.category_volume(EarconCategory::Navigation), 0.3);
    }
    
    #[test]
    fn test_event_categories() {
        assert_eq!(EarconEvent::ZoomIn.category(), EarconCategory::Navigation);
        assert_eq!(EarconEvent::CommandAccepted.category(), EarconCategory::CommandFeedback);
        assert_eq!(EarconEvent::Notification.category(), EarconCategory::SystemStatus);
        assert_eq!(EarconEvent::UserJoined.category(), EarconCategory::Collaboration);
        assert_eq!(EarconEvent::BezelExpand.category(), EarconCategory::BezelUi);
    }
    
    #[test]
    fn test_event_priorities() {
        // Critical events should have highest priority
        assert_eq!(EarconEvent::DangerousCommandWarning.priority(), 10);
        assert_eq!(EarconEvent::TacticalAlert.priority(), 10);
        
        // Low priority events
        assert!(EarconEvent::ButtonHover.priority() < EarconEvent::CommandError.priority());
    }
    
    #[test]
    fn test_spatial_position() {
        let center = SpatialPosition::center();
        assert_eq!(center.pan(), 0.0);
        assert_eq!(center.attenuation(), 1.0);
        
        let left = SpatialPosition::from_sector_position(-0.5, 0.0, 0.0);
        assert_eq!(left.pan(), -0.5);
        
        let far = SpatialPosition::from_sector_position(0.0, 0.0, 1.0);
        assert!(far.attenuation() < 1.0);
    }
    
    #[test]
    fn test_mute_unmute() {
        let mut player = EarconPlayer::new();
        assert!(player.is_enabled());
        
        player.mute();
        assert!(!player.is_enabled());
        
        player.unmute();
        assert!(player.is_enabled());
    }
    
    #[test]
    fn test_concurrent_limit() {
        let mut player = EarconPlayer::new();
        player.set_max_concurrent(2);
        
        // Simulate filling up concurrent slots
        player.active_sounds.push(EarconEvent::ZoomIn);
        player.active_sounds.push(EarconEvent::ZoomOut);
        
        assert_eq!(player.active_sound_count(), 2);
        
        player.clear_active_sounds();
        assert_eq!(player.active_sound_count(), 0);
    }
    
    #[test]
    fn test_reset_to_defaults() {
        let mut player = EarconPlayer::new();
        
        player.set_master_volume(0.5);
        player.set_enabled(false);
        player.mute();
        
        player.reset_to_defaults();
        
        assert_eq!(player.master_volume(), 1.0);
        assert!(player.is_enabled());
    }
}
