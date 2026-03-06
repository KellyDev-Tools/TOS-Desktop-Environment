
pub mod logger;
pub mod settings;
pub mod audio;
pub mod marketplace;
pub mod ai;
pub mod search;
pub mod haptic;
pub mod portal;
pub mod priority;
pub mod registry;
pub mod session;
pub mod trust;
pub mod heuristic;

pub use logger::LoggerService;
pub use settings::SettingsService;
pub use audio::AudioService;
pub use marketplace::MarketplaceService;
pub use ai::AiService;
pub use search::SearchService;
pub use haptic::HapticService;
pub use portal::PortalService;
pub use priority::PriorityService;
pub use registry::ServiceRegistry;
pub use session::SessionService;
pub use trust::TrustService;
pub use heuristic::HeuristicService;

use std::sync::{Arc, Mutex};

pub struct ServiceManager {
    pub logger: Arc<LoggerService>,
    pub settings: Arc<SettingsService>,
    pub audio: Arc<AudioService>,
    pub ai: Arc<AiService>,
    pub search: Arc<SearchService>,
    pub haptic: Arc<HapticService>,
    pub portal: Arc<PortalService>,
    pub priority: Arc<PriorityService>,
    pub registry: Arc<Mutex<ServiceRegistry>>,
    pub session: Arc<SessionService>,
    pub trust: Arc<TrustService>,
    pub heuristic: Arc<HeuristicService>,
    pub marketplace: Arc<MarketplaceService>,
}

impl ServiceManager {
    pub fn new() -> Self {
        let logger = Arc::new(LoggerService::new());
        let settings = Arc::new(SettingsService::new());

        // Read anchor port from settings, default 7000.
        let anchor_port: u16 = settings.default_settings_public()
            .global.get("tos.network.anchor_port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(7000);
        let registry = Arc::new(Mutex::new(ServiceRegistry::new(anchor_port)));

        let (audio_svc, audio_warning) = AudioService::new();
        let audio = Arc::new(audio_svc);
        let ai = Arc::new(AiService::new());
        let search = Arc::new(SearchService::new(registry.clone()));
        let haptic = Arc::new(HapticService::new());
        let portal = Arc::new(PortalService::new());
        let priority = Arc::new(PriorityService::new());

        let session = Arc::new(SessionService::new(registry.clone()));
        let trust = Arc::new(TrustService::new());
        let heuristic = Arc::new(HeuristicService::new(registry.clone()));
        let marketplace = Arc::new(MarketplaceService::new(registry.clone()));
        
        // Establish cross-service dependencies (e.g., logging triggers audio cues)
        logger.set_audio_service(audio.clone());

        // Surface any init warnings through the logger
        if let Some(warning) = audio_warning {
            logger.log(&warning, 2);
        }
        
        Self {
            logger,
            settings,
            audio,
            ai,
            search,
            haptic,
            portal,
            priority,
            registry: registry.clone(),
            session,
            trust,
            heuristic,
            marketplace,
        }
    }
    pub fn set_ipc(&self, ipc: std::sync::Arc<dyn crate::common::ipc_dispatcher::IpcDispatcher>) {
        self.logger.set_ipc(ipc.clone());
        self.ai.set_ipc(ipc);
    }
}
