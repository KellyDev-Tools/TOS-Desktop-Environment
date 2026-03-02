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

use std::sync::Arc; // Mutex unused after decoupling
// use crate::common::TosState; // Unused after decoupling

pub struct ServiceManager {
    pub logger: Arc<LoggerService>,
    pub settings: Arc<SettingsService>,
    pub audio: Arc<AudioService>,
    pub ai: Arc<AiService>,
}

impl ServiceManager {
    pub fn new() -> Self {
        let logger = Arc::new(LoggerService::new());
        let settings = Arc::new(SettingsService::new());
        let audio = Arc::new(AudioService::new());
        let ai = Arc::new(AiService::new());
        
        // Establish cross-service dependencies (e.g., logging triggers audio cues)
        logger.set_audio_service(audio.clone());
        
        Self {
            logger,
            settings,
            audio,
            ai,
        }
    }

    pub fn set_ipc(&self, ipc: std::sync::Arc<dyn crate::common::ipc_dispatcher::IpcDispatcher>) {
        self.logger.set_ipc(ipc.clone());
        self.ai.set_ipc(ipc);
    }
}
