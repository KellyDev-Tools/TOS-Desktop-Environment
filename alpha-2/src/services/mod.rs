pub mod logger;
pub mod settings;
pub mod audio;

pub use logger::LoggerService;
pub use settings::SettingsService;
pub use audio::AudioService;

use std::sync::{Arc, Mutex};
use crate::common::TosState;

pub struct ServiceManager {
    pub logger: Arc<LoggerService>,
    pub settings: Arc<SettingsService>,
    pub audio: Arc<AudioService>,
}

impl ServiceManager {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        Self {
            logger: Arc::new(LoggerService::new(state.clone())),
            settings: Arc::new(SettingsService::new(state.clone())),
            audio: Arc::new(AudioService::new(state.clone())),
        }
    }
}
