pub mod ai;
pub mod audio;
pub mod capture;
pub mod haptic;
pub mod heuristic;
pub mod logger;
pub mod marketplace;
pub mod portal;
pub mod priority;
pub mod registry;
pub mod search;
pub mod session;
pub mod settings;
pub mod trust;

pub use ai::AiService;
pub use audio::AudioService;
pub use capture::CaptureService;
pub use haptic::HapticService;
pub use heuristic::HeuristicService;
pub use logger::LoggerService;
pub use marketplace::MarketplaceService;
pub use portal::PortalService;
pub use priority::PriorityService;
pub use registry::ServiceRegistry;
pub use search::SearchService;
pub use session::SessionService;
pub use settings::SettingsService;
pub use trust::TrustService;

use crate::config::TosConfig;
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
    pub capture: Arc<CaptureService>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self::with_config(&TosConfig::default())
    }

    pub fn with_config(config: &TosConfig) -> Self {
        // 1. Initialize Registry first, as it is the discovery backbone (§4.1)
        let anchor_port = config.remote.anchor_port;
        let registry = Arc::new(Mutex::new(ServiceRegistry::new(anchor_port)));

        // 2. Initialize Services with Registry awareness
        let settings = Arc::new(SettingsService::with_registry_and_config(
            registry.clone(),
            config,
        ));
        let logger = Arc::new(LoggerService::with_registry(registry.clone()));

        let (audio_svc, audio_warning) = AudioService::new();
        let audio = Arc::new(audio_svc);
        let ai = Arc::new(AiService::new());
        let search = Arc::new(SearchService::new(registry.clone()));
        let haptic = Arc::new(HapticService::new());
        let portal = Arc::new(PortalService::new());
        let priority = Arc::new(PriorityService::new(registry.clone()));

        let session = Arc::new(SessionService::with_config(registry.clone(), config));
        let trust = Arc::new(TrustService::new());
        let heuristic = Arc::new(HeuristicService::new(registry.clone()));
        let marketplace = Arc::new(MarketplaceService::new(registry.clone()));

        let capture_svc = CaptureService::new();
        capture_svc.set_backend(Arc::new(capture::MockCaptureBackend));
        let capture = Arc::new(capture_svc);

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
            capture,
        }
    }
    pub fn set_ipc(&self, ipc: std::sync::Arc<dyn crate::ipc::IpcDispatcher>) {
        self.logger.set_ipc(ipc.clone());
        self.ai.set_ipc(ipc);
    }
}
