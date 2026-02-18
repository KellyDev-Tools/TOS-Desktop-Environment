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
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

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
#[derive(Debug)]
pub struct AudioManager {
    /// Active ambience per sector
    pub sector_ambience: HashMap<Uuid, AmbienceProfile>,
    /// Global volume
    pub volume: f32,
    /// Whether audio is muted
    pub muted: bool,
    /// Rodio output stream
    #[cfg(feature = "accessibility")]
    _stream: Option<OutputStream>,
    /// Rodio output stream handle
    #[cfg(feature = "accessibility")]
    stream_handle: Option<OutputStreamHandle>,
    /// Active ambience sinks
    #[cfg(feature = "accessibility")]
    ambience_sinks: HashMap<Uuid, Vec<Sink>>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    pub fn new() -> Self {
        #[cfg(feature = "accessibility")]
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };

        Self {
            sector_ambience: HashMap::new(),
            volume: 0.8,
            muted: false,
            #[cfg(feature = "accessibility")]
            _stream: stream,
            #[cfg(feature = "accessibility")]
            stream_handle: handle,
            #[cfg(feature = "accessibility")]
            ambience_sinks: HashMap::new(),
        }
    }

    pub fn play_event(&self, event: AudioEvent) {
        if self.muted { return; }
        tracing::info!("TOS // AUDIO EVENT: {:?}", event);
        
        #[cfg(feature = "accessibility")]
        if let Some(ref handle) = self.stream_handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume);
                
                let freq = match event {
                    AudioEvent::AmbientHum => 60.0,
                    AudioEvent::BridgeChirps => 2000.0,
                    AudioEvent::ComputerThinking => 800.0,
                    AudioEvent::DataTransfer => 1200.0,
                    AudioEvent::SectorTransition => 150.0,
                    AudioEvent::PortalHum => 40.0,
                    AudioEvent::AlertBeep => 1000.0,
                };

                let source = rodio::source::SineWave::new(freq)
                    .take_duration(std::time::Duration::from_millis(200))
                    .amplify(0.5);
                
                sink.append(source);
                sink.detach();
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
        if self.muted { return; }
        
        // Stop existing sinks for this sector
        if let Some(sinks) = self.ambience_sinks.remove(&sector_id) {
            for sink in sinks {
                sink.stop();
            }
        }

        let mut sinks = Vec::new();
        if let Some(ref handle) = self.stream_handle {
            // Base layer
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(profile.volume * self.volume);
                let freq = match profile.base_loop {
                    AudioEvent::AmbientHum => 55.0,
                    _ => 60.0,
                };
                let source = rodio::source::SineWave::new(freq).repeat_infinite();
                sink.append(source);
                sinks.push(sink);
            }

            // Secondary layers
            for layer in profile.secondary_layers {
                if let Ok(sink) = Sink::try_new(handle) {
                    sink.set_volume(profile.volume * self.volume * 0.5);
                    let freq = match layer {
                        AudioEvent::BridgeChirps => 2500.0,
                        _ => 1000.0,
                    };
                    let source = rodio::source::SineWave::new(freq).repeat_infinite();
                    sink.append(source);
                    sinks.push(sink);
                }
            }
        }
        
        self.ambience_sinks.insert(sector_id, sinks);
    }
}
