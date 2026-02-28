pub mod logger;
pub mod settings;
pub mod audio;
pub mod marketplace;
pub mod ai;

pub use logger::LoggerService;
pub use settings::SettingsService;
pub use audio::AudioService;
pub use marketplace::MarketplaceService;
pub use ai::AiService;

use std::sync::{Arc, Mutex};
use crate::common::TosState;

pub struct ServiceManager {
    pub logger: Arc<LoggerService>,
    pub settings: Arc<SettingsService>,
    pub audio: Arc<AudioService>,
    pub ai: Arc<AiService>,
}

impl ServiceManager {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        let logger = Arc::new(LoggerService::new(state.clone()));
        let settings = Arc::new(SettingsService::new(state.clone()));
        let audio = Arc::new(AudioService::new(state.clone()));
        let ai = Arc::new(AiService::new(state.clone()));
        
        // Establish cross-service dependencies (e.g., logging triggers audio cues)
        logger.set_audio_service(audio.clone());
        
        Self {
            logger,
            settings,
            audio,
            ai,
        }
    }
}
