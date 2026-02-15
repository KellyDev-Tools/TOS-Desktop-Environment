//! General Purpose Audio Manager
//! 
//! Handles background ambience, UI sounds, and sector-specific spatial audio.

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
#[derive(Debug, Default)]
pub struct AudioManager {
    /// Active ambience per sector
    pub sector_ambience: HashMap<Uuid, AmbienceProfile>,
    /// Global volume
    pub volume: f32,
    /// Whether audio is muted
    pub muted: bool,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            sector_ambience: HashMap::new(),
            volume: 0.8,
            muted: false,
        }
    }

    pub fn play_event(&self, event: AudioEvent) {
        if self.muted { return; }
        tracing::info!("TOS // AUDIO EVENT: {:?}", event);
        // Real implementation would trigger rodio sinks here
    }

    pub fn set_sector_ambience(&mut self, sector_id: Uuid, profile: AmbienceProfile) {
        self.sector_ambience.insert(sector_id, profile);
        tracing::info!("TOS // UPDATED AMBIENCE FOR SECTOR: {}", sector_id);
    }
}
