//! General Purpose Audio Manager
//!
//! Handles background ambience, UI sounds, and sector-specific spatial audio.
//! Uses a three-bus Kira hierarchy:
//!   Master
//!     ├── Ambience Bus  (looping backgrounds, with low-pass filter for L3 focus)
//!     ├── UI Bus        (earcon one-shots)
//!     └── Voice/TTS Bus (high-priority speech / TTS one-shots)

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

/// Whether an event is a high-priority critical alert that should duck ambience.
impl AudioEvent {
    fn is_critical(self) -> bool {
        matches!(self, AudioEvent::AlertBeep)
    }
}

#[cfg(feature = "accessibility")]
use kira::{
    track::{TrackBuilder, TrackHandle},
    sound::static_sound::{StaticSoundData, StaticSoundSettings, StaticSoundHandle},
    tween::Tween,
    effect::{
        filter::{FilterBuilder, FilterHandle, FilterMode},
        volume_control::{VolumeControlBuilder, VolumeControlHandle},
    },
    Volume,
};

// In test builds (or with --features test-audio) swap to Kira's zero-hardware MockBackend
// so that `cargo test` never probes ALSA/PulseAudio devices. Production builds use CpalBackend.
#[cfg(all(feature = "accessibility", any(test, feature = "test-audio")))]
use kira::manager::{backend::mock::MockBackend as KiraBackend, AudioManager as KiraManager, AudioManagerSettings};
#[cfg(all(feature = "accessibility", not(any(test, feature = "test-audio"))))]
use kira::manager::{backend::cpal::CpalBackend as KiraBackend, AudioManager as KiraManager, AudioManagerSettings};


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

/// Audio Manager — three-bus Kira hierarchy.
pub struct AudioManager {
    /// Active ambience per sector
    pub sector_ambience: HashMap<Uuid, AmbienceProfile>,
    /// Global volume
    pub volume: f32,
    /// Whether audio is muted
    pub muted: bool,

    // ── Kira backend ─────────────────────────────────────────────────────────
    /// Kira AudioManager instance
    #[cfg(feature = "accessibility")]
    manager: Option<KiraManager<KiraBackend>>,

    // ── Mixer buses ──────────────────────────────────────────────────────────
    /// Ambience bus: looping backgrounds with optional low-pass filter
    #[cfg(feature = "accessibility")]
    ambience_track: Option<TrackHandle>,
    /// Ambience bus low-pass filter handle (tweaked during L3 Application Focus)
    #[cfg(feature = "accessibility")]
    ambience_filter: Option<FilterHandle>,
    /// Ambience bus volume-control handle (used for ducking under critical alerts)
    #[cfg(feature = "accessibility")]
    ambience_volume_control: Option<VolumeControlHandle>,

    /// UI bus: earcon one-shots
    #[cfg(feature = "accessibility")]
    ui_track: Option<TrackHandle>,

    /// Voice/TTS bus: high-priority speech and TTS one-shots
    #[cfg(feature = "accessibility")]
    voice_track: Option<TrackHandle>,

    // ── State ────────────────────────────────────────────────────────────────
    /// Active ambience sound handles (for smooth fade-out on sector change)
    #[cfg(feature = "accessibility")]
    active_ambience: HashMap<Uuid, Vec<StaticSoundHandle>>,
    /// Whether the ambience bus is currently ducked
    #[cfg(feature = "accessibility")]
    ambience_ducked: bool,
}

// Custom Debug — Kira handles don't implement Debug
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
        // ── Kira initialisation ───────────────────────────────────────────────
        #[cfg(feature = "accessibility")]
        let mut manager = match KiraManager::<KiraBackend>::new(AudioManagerSettings::default()) {
            Ok(m) => Some(m),
            Err(e) => {
                tracing::error!("Failed to initialize Kira AudioManager: {:?}", e);
                None
            }
        };

        // ── Bus construction ──────────────────────────────────────────────────
        #[cfg(feature = "accessibility")]
        let mut ui_track = None;
        #[cfg(feature = "accessibility")]
        let mut ambience_track = None;
        #[cfg(feature = "accessibility")]
        let mut ambience_filter: Option<FilterHandle> = None;
        #[cfg(feature = "accessibility")]
        let mut ambience_volume_control: Option<VolumeControlHandle> = None;
        #[cfg(feature = "accessibility")]
        let mut voice_track = None;

        #[cfg(feature = "accessibility")]
        if let Some(ref mut m) = manager {
            // -- Ambience Bus: add a low-pass filter + volume-control effect so we
            //    can: (a) muffle it during L3 ApplicationFocus and (b) duck it under alerts.
            let mut ambience_builder = TrackBuilder::default();
            ambience_filter = Some(
                ambience_builder.add_effect(FilterBuilder::new().mode(FilterMode::LowPass).cutoff(20_000.0))
            );
            ambience_volume_control = Some(
                ambience_builder.add_effect(VolumeControlBuilder::new(Volume::Amplitude(1.0)))
            );
            ambience_track = m.add_sub_track(ambience_builder).ok();

            // -- UI Bus: plain sub-track for one-shot earcons
            ui_track = m.add_sub_track(TrackBuilder::default()).ok();

            // -- Voice/TTS Bus: dedicated sub-track for all TTS and voice audio
            voice_track = m.add_sub_track(TrackBuilder::default()).ok();
        }

        Self {
            sector_ambience: HashMap::new(),
            volume: 0.8,
            muted: false,
            #[cfg(feature = "accessibility")]
            manager,
            #[cfg(feature = "accessibility")]
            ambience_track,
            #[cfg(feature = "accessibility")]
            ambience_filter,
            #[cfg(feature = "accessibility")]
            ambience_volume_control,
            #[cfg(feature = "accessibility")]
            ui_track,
            #[cfg(feature = "accessibility")]
            voice_track,
            #[cfg(feature = "accessibility")]
            active_ambience: HashMap::new(),
            #[cfg(feature = "accessibility")]
            ambience_ducked: false,
        }
    }

    /// Play an audio event through the UI bus.
    /// Critical events (AlertBeep) automatically duck the ambience bus.
    pub fn play_event(&mut self, event: AudioEvent) {
        if self.muted { return; }
        tracing::info!("TOS // AUDIO EVENT: {:?}", event);

        // Duck ambience for critical alerts, then unduck after a delay
        if event.is_critical() {
            self.duck_ambience();
        }

        #[cfg(feature = "accessibility")]
        if let (Some(ref mut manager), Some(ref track)) = (&mut self.manager, &self.ui_track) {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let sound_path = format!("{}/.local/share/tos/audio/{:?}.wav", home, event).to_lowercase();

            if std::path::Path::new(&sound_path).exists() {
                if let Ok(data) = StaticSoundData::from_file(&sound_path) {
                    let mut settings = StaticSoundSettings::new()
                        .output_destination(track)
                        .volume(self.volume as f64);
                    settings.fade_in_tween = Some(Tween::default());
                    let _ = manager.play(data.with_settings(settings));
                }
            } else {
                tracing::debug!(
                    "Audio file not found for event {:?}, skipping synthesized fallback in Kira transition.",
                    event
                );
            }
        }
    }

    /// Play a one-shot through the high-priority Voice/TTS bus.
    /// Use for speech synthesis output or TTS earcons that must not be overshadowed.
    pub fn play_voice_event(&mut self, event: AudioEvent) {
        if self.muted { return; }
        tracing::info!("TOS // VOICE AUDIO EVENT: {:?}", event);

        #[cfg(feature = "accessibility")]
        if let (Some(ref mut manager), Some(ref track)) = (&mut self.manager, &self.voice_track) {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let sound_path = format!("{}/.local/share/tos/audio/{:?}.wav", home, event).to_lowercase();

            if std::path::Path::new(&sound_path).exists() {
                if let Ok(data) = StaticSoundData::from_file(&sound_path) {
                    let mut settings = StaticSoundSettings::new()
                        .output_destination(track)
                        .volume(self.volume as f64);
                    settings.fade_in_tween = Some(Tween::default());
                    let _ = manager.play(data.with_settings(settings));
                }
            }
        }
    }

    /// Duck the ambience bus volume to ~20% for alert clarity (side-chain effect).
    /// The caller should call unduck_ambience() after the alert finishes.
    pub fn duck_ambience(&mut self) {
        #[cfg(feature = "accessibility")]
        {
            if self.ambience_ducked { return; }
            if let Some(ref mut vc) = self.ambience_volume_control {
                // Tween the volume down to 0.2 (20%) smoothly
                let duck_tween = Tween {
                    duration: std::time::Duration::from_millis(80),
                    ..Default::default()
                };
                let _ = vc.set_volume(Volume::Amplitude(0.2), duck_tween);
                self.ambience_ducked = true;
                tracing::debug!("TOS // AUDIO: Ambience bus ducked for critical alert.");
            }
        }
    }

    /// Restore ambience bus to full volume after a critical event.
    pub fn unduck_ambience(&mut self) {
        #[cfg(feature = "accessibility")]
        {
            if !self.ambience_ducked { return; }
            if let Some(ref mut vc) = self.ambience_volume_control {
                let unduck_tween = Tween {
                    duration: std::time::Duration::from_millis(600),
                    ..Default::default()
                };
                let _ = vc.set_volume(Volume::Amplitude(1.0), unduck_tween);
                self.ambience_ducked = false;
                tracing::debug!("TOS // AUDIO: Ambience bus volume restored.");
            }
        }
    }

    /// Lower the ambience bus low-pass filter cutoff for Level 3 Application Focus.
    /// This makes background sounds feel "distant" when the user is deep in an application.
    pub fn apply_focus_filter(&mut self) {
        #[cfg(feature = "accessibility")]
        if let Some(ref mut filter) = self.ambience_filter {
            let tween = Tween {
                duration: std::time::Duration::from_millis(400),
                ..Default::default()
            };
            // Drop to ~800 Hz to give a muffled / "in the background" feel
            let _ = filter.set_cutoff(800.0, tween);
            tracing::debug!("TOS // AUDIO: L3 focus low-pass filter applied to ambience bus.");
        }
    }

    /// Restore the ambience bus filter cutoff to the full-range pass-through value.
    pub fn remove_focus_filter(&mut self) {
        #[cfg(feature = "accessibility")]
        if let Some(ref mut filter) = self.ambience_filter {
            let tween = Tween {
                duration: std::time::Duration::from_millis(400),
                ..Default::default()
            };
            let _ = filter.set_cutoff(20_000.0, tween);
            tracing::debug!("TOS // AUDIO: L3 focus low-pass filter removed from ambience bus.");
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
            let base_path = format!(
                "{}/.local/share/tos/audio/{:?}.wav", home, profile.base_loop
            ).to_lowercase();
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
                let layer_path = format!(
                    "{}/.local/share/tos/audio/{:?}.wav", home, layer
                ).to_lowercase();
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

    /// Play a spatial earcon (higher-level mixer layer).
    /// Currently maps to a volume-attenuated UI bus sound; full Kira SpatialEmitter
    /// wiring is in Phase 3.2 (see earcons.rs for the EarconPlayer path).
    pub fn play_spatial_earcon(&mut self, event: AudioEvent, _x: f32, _y: f32, _z: f32) {
        tracing::debug!(
            "TOS // SPATIAL AUDIO (Kira-Ready): {:?} at ({}, {}, {})",
            event, _x, _y, _z
        );
        self.play_event(event);
    }
}
