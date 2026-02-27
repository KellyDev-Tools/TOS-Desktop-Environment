pub mod logger;
pub mod settings;
pub mod audio;
pub mod marketplace;

pub use logger::LoggerService;
pub use settings::SettingsService;
pub use audio::AudioService;
pub use marketplace::MarketplaceService;

use std::sync::{Arc, Mutex};
use crate::common::TosState;

pub struct ServiceManager {
    pub logger: Arc<LoggerService>,
    pub settings: Arc<SettingsService>,
    pub audio: Arc<AudioService>,
}

impl ServiceManager {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        let logger = Arc::new(LoggerService::new(state.clone()));
        let settings = Arc::new(SettingsService::new(state.clone()));
        let audio = Arc::new(AudioService::new(state.clone()));
        
        // Link services §21.2
        logger.set_audio_service(audio.clone());
        
        Self {
            logger,
            settings,
            audio,
        }
    }
}

