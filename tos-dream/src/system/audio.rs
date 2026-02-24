//! General Purpose Audio Manager
//! 
//! Handles background ambience, UI sounds, and sector-specific spatial audio.

// Auditory Interface submodules
pub mod earcons;
pub mod themes;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Audio event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioEvent {
    AmbientHum,
    BridgeChirps,
    ComputerThinking,
    DataTransfer,
    SectorTransition,
    PortalHum,
    AlertBeep,
}

#[cfg(feature = "accessibility")]
use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager as KiraManager, AudioManagerSettings},
    track::{TrackBuilder, TrackHandle},
    sound::static_sound::{StaticSoundData, StaticSoundSettings, StaticSoundHandle},
    tween::Tween,
};

/// Ambience profile for sectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbienceProfile {
    pub base_loop: AudioEvent,
    pub secondary_layers: Vec<AudioEvent>,
    pub volume: f32,
    pub pitch: f32,
}

impl Default for AmbienceProfile {
    fn default() -> Self {
        Self {
            base_loop: AudioEvent::AmbientHum,
            secondary_layers: vec![AudioEvent::BridgeChirps],
            volume: 0.3,
            pitch: 1.0,
        }
    }
}

/// Audio Manager
pub struct AudioManager {
    /// Active ambience per sector
    pub sector_ambience: HashMap<Uuid, AmbienceProfile>,
    /// Global volume
    pub volume: f32,
    /// Whether audio is muted
    pub muted: bool,
    
    /// Kira manager
    #[cfg(feature = "accessibility")]
    manager: Option<KiraManager<CpalBackend>>,
    
    /// Mixer buses
    #[cfg(feature = "accessibility")]
    ui_track: Option<TrackHandle>,
    #[cfg(feature = "accessibility")]
    ambience_track: Option<TrackHandle>,
    
    /// Active sound handles (for stopping/modulating)
    #[cfg(feature = "accessibility")]
    active_ambience: HashMap<Uuid, Vec<StaticSoundHandle>>,
}

// Custom Debug implementation since Kira handles aren't Debug
impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
            .field("sector_ambience", &self.sector_ambience)
            .field("volume", &self.volume)
            .field("muted", &self.muted)
            .finish()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    pub fn new() -> Self {
        #[cfg(feature = "accessibility")]
        let mut manager = match KiraManager::<CpalBackend>::new(AudioManagerSettings::default()) {
            Ok(m) => Some(m),
            Err(e) => {
                tracing::error!("Failed to initialize Kira AudioManager: {}", e);
                None
            }
        };

        #[cfg(feature = "accessibility")]
        let mut ui_track = None;
        #[cfg(feature = "accessibility")]
        let mut ambience_track = None;

        #[cfg(feature = "accessibility")]
        if let Some(ref mut m) = manager {
            ui_track = m.add_sub_track(TrackBuilder::default()).ok();
            ambience_track = m.add_sub_track(TrackBuilder::default()).ok();
        }

        Self {
            sector_ambience: HashMap::new(),
            volume: 0.8,
            muted: false,
            #[cfg(feature = "accessibility")]
            manager,
            #[cfg(feature = "accessibility")]
            ui_track,
            #[cfg(feature = "accessibility")]
            ambience_track,
            #[cfg(feature = "accessibility")]
            active_ambience: HashMap::new(),
        }
    }

    pub fn play_event(&mut self, event: AudioEvent) {
        if self.muted { return; }
        tracing::info!("TOS // AUDIO EVENT: {:?}", event);
        
        #[cfg(feature = "accessibility")]
        if let (Some(ref mut manager), Some(ref track)) = (&mut self.manager, &self.ui_track) {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let sound_path = format!("{}/.local/share/tos/audio/{:?}.wav", home, event).to_lowercase();
            
            if std::path::Path::new(&sound_path).exists() {
                if let Ok(data) = StaticSoundData::from_file(&sound_path) {
                    let mut settings = StaticSoundSettings::new()
                        .output_destination(track)
                        .volume(self.volume as f64);
                    
                    // Add a subtle fade-in for tactical smoothness
                    settings.fade_in_tween = Some(Tween::default());
                    
                    let _ = manager.play(data.with_settings(settings));
                }
            } else {
                tracing::debug!("Audio file not found for event {:?}, skipping synthesized fallback in Kira transition.", event);
            }
        }
    }

    pub fn set_sector_ambience(&mut self, sector_id: Uuid, profile: AmbienceProfile) {
        self.sector_ambience.insert(sector_id, profile.clone());
        tracing::info!("TOS // UPDATED AMBIENCE FOR SECTOR: {}", sector_id);
        
        #[cfg(feature = "accessibility")]
        self.update_ambience_sinks(sector_id, profile);
    }

    #[cfg(feature = "accessibility")]
    fn update_ambience_sinks(&mut self, sector_id: Uuid, profile: AmbienceProfile) {
        // Stop existing ambience for this sector with a smooth fade-out
        if let Some(handles) = self.active_ambience.remove(&sector_id) {
            for mut handle in handles {
                let _ = handle.stop(Tween::default());
            }
        }

        if self.muted { return; }

        let mut new_handles = Vec::new();
        if let (Some(ref mut manager), Some(ref track)) = (&mut self.manager, &self.ambience_track) {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            
            // Base layer
            let base_path = format!("{}/.local/share/tos/audio/{:?}.wav", home, profile.base_loop).to_lowercase();
            if std::path::Path::new(&base_path).exists() {
                if let Ok(data) = StaticSoundData::from_file(&base_path) {
                    let mut settings = StaticSoundSettings::new()
                        .output_destination(track)
                        .volume(profile.volume as f64 * self.volume as f64)
                        .loop_region(0.0..);
                    
                    settings.fade_in_tween = Some(Tween::default());
                    
                    if let Ok(handle) = manager.play(data.with_settings(settings)) {
                        new_handles.push(handle);
                    }
                }
            }

            // Secondary layers
            for layer in profile.secondary_layers {
                let layer_path = format!("{}/.local/share/tos/audio/{:?}.wav", home, layer).to_lowercase();
                if std::path::Path::new(&layer_path).exists() {
                   if let Ok(data) = StaticSoundData::from_file(&layer_path) {
                        let mut settings = StaticSoundSettings::new()
                            .output_destination(track)
                            .volume(profile.volume as f64 * self.volume as f64 * 0.5)
                            .loop_region(0.0..);
                        
                        settings.fade_in_tween = Some(Tween::default());
                        
                        if let Ok(handle) = manager.play(data.with_settings(settings)) {
                            new_handles.push(handle);
                        }
                    }
                }
            }
        }
        
        self.active_ambience.insert(sector_id, new_handles);
    }

    /// Play a spatial earcon (higher-level mixer layer)
    pub fn play_spatial_earcon(&mut self, event: AudioEvent, _x: f32, _y: f32, _z: f32) {
        tracing::debug!("TOS // SPATIAL AUDIO (Kira-Ready): {:?} at ({}, {}, {})", event, _x, _y, _z);
        self.play_event(event);
    }
}
