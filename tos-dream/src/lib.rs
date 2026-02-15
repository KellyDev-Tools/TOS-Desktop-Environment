pub mod system;
pub mod ui;
pub mod modules;
pub mod marketplace;
pub mod cli;

#[cfg(feature = "accessibility")]
pub mod accessibility;

use system::input::SemanticEvent;
use modules::{ModuleRegistry, ModuleState, ModuleManifest};
use serde::{Deserialize, Serialize};

// Phase 11 imports
use system::reset::TacticalReset;
use system::voice::VoiceCommandProcessor;
use system::shell_api::ShellApi;
use system::security::SecurityManager;
use ui::minimap::MiniMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchyLevel {
    GlobalOverview,
    CommandHub,
    ApplicationFocus,
    SplitView,
    DetailInspector,
    BufferInspector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub id: uuid::Uuid,
    pub sector_index: usize,
    pub hub_index: usize,
    pub current_level: HierarchyLevel,
    pub active_app_index: Option<usize>,
    pub bezel_expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandHubMode {
    Command,
    Directory,
    Activity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub name: String,
    pub color: String,
    pub role: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Local,
    TOSNative,
    SSH,
    HTTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: uuid::Uuid,
    pub name: String,
    pub color: String,
    pub hubs: Vec<CommandHub>,
    pub active_hub_index: usize,
    pub host: String,
    pub connection_type: ConnectionType,
    pub participants: Vec<Participant>,
    pub portal_active: bool,
    pub portal_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHub {
    pub id: uuid::Uuid,
    pub mode: CommandHubMode,
    pub prompt: String,
    pub applications: Vec<Application>,
    pub active_app_index: Option<usize>,
    pub terminal_output: Vec<String>,
    pub confirmation_required: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMode {
    Full,       // Active level, full interactivity and animations
    Throttled,  // Background level, reduced frame rate/simple styles
    Static,     // Distant level, rendered as a static texture-like state
}

impl RenderMode {
    pub fn throttle(self) -> Self {
        match self {
            RenderMode::Full => RenderMode::Throttled,
            RenderMode::Throttled => RenderMode::Static,
            RenderMode::Static => RenderMode::Static,
        }
    }
}

pub trait ApplicationModel: std::fmt::Debug + Send + Sync {
    fn title(&self) -> String;
    fn app_class(&self) -> String;
    fn bezel_actions(&self) -> Vec<String>;
    fn handle_command(&self, cmd: &str) -> Option<String>;
}

pub trait SectorType: std::fmt::Debug + Send + Sync {
    fn name(&self) -> String;
    fn command_favourites(&self) -> Vec<String>;
    fn default_hub_mode(&self) -> CommandHubMode;
}

pub trait TosModule: std::fmt::Debug + Send + Sync {
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn on_load(&mut self, _state: &mut TosState) {}
    fn on_unload(&mut self, _state: &mut TosState) {}
    fn render_override(&self, _level: HierarchyLevel) -> Option<String> { None }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: uuid::Uuid,
    pub title: String,
    pub app_class: String,
    pub is_minimized: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TosState {
    pub current_level: HierarchyLevel,
    pub sectors: Vec<Sector>,
    pub viewports: Vec<Viewport>,
    pub active_viewport_index: usize,
    pub escape_count: usize, // For Tactical Reset
    pub fps: f32,
    pub performance_alert: bool,
    #[serde(skip)]
    pub modules: Vec<Box<dyn TosModule>>,
    pub portal_security_bypass: bool,
    pub approval_requested_sector: Option<uuid::Uuid>,
    /// Module registry for Phase 8
    #[serde(skip)]
    pub module_registry: ModuleRegistry,
    /// Application model registry
    #[serde(skip)]
    pub app_model_registry: modules::app_model::AppModelRegistry,
    /// Sector type registry
    #[serde(skip)]
    pub sector_type_registry: modules::sector_type::SectorTypeRegistry,
    /// Marketplace for Phase 9
    #[serde(skip)]
    pub marketplace: marketplace::Marketplace,
    /// Accessibility manager for Phase 10
    #[serde(skip)]
    #[cfg(feature = "accessibility")]
    pub accessibility: Option<accessibility::AccessibilityManager>,
    /// Live feed server for Phase 10
    #[serde(skip)]
    #[cfg(feature = "live-feed")]
    pub live_feed: Option<system::live_feed::LiveFeedServer>,
    /// Phase 11: Tactical Mini-Map
    #[serde(skip)]
    pub minimap: MiniMap,
    /// Phase 11: Tactical Reset
    #[serde(skip)]
    pub tactical_reset: TacticalReset,
    /// Phase 11: Voice Command Processor
    #[serde(skip)]
    pub voice: VoiceCommandProcessor,
    /// Phase 11: Shell API
    #[serde(skip)]
    pub shell_api: ShellApi,
    /// Phase 11: Security Manager
    #[serde(skip)]
    pub security: SecurityManager,
}

impl std::fmt::Debug for TosState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TosState")
            .field("current_level", &self.current_level)
            .field("sectors", &self.sectors)
            .field("viewports", &self.viewports)
            .field("active_viewport_index", &self.active_viewport_index)
            .field("escape_count", &self.escape_count)
            .field("fps", &self.fps)
            .field("performance_alert", &self.performance_alert)
            .field("modules", &self.modules.len())
            .field("portal_security_bypass", &self.portal_security_bypass)
            .field("approval_requested_sector", &self.approval_requested_sector)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct EngineeringModule {
    pub power_distribution: [u8; 3], // Propulsion, Shields, Sensors
}

impl TosModule for EngineeringModule {
    fn name(&self) -> String { "ENGINEERING_CORE".to_string() }
    fn version(&self) -> String { "1.2.0".to_string() }
    fn on_load(&mut self, _state: &mut TosState) {
        println!("ENGINEERING MODULE LOADED: PRIMARY CORE SYNCED");
    }
    fn render_override(&self, level: HierarchyLevel) -> Option<String> {
        if level == HierarchyLevel::ApplicationFocus {
            Some(format!(
                r#"<div class="engineering-overlay">
                    <div class="eng-stat">PROPULSION: {}%</div>
                    <div class="eng-stat">SHIELDS: {}%</div>
                    <div class="eng-stat">SENSORS: {}%</div>
                </div>"#,
                self.power_distribution[0], self.power_distribution[1], self.power_distribution[2]
            ))
        } else {
            None
        }
    }
}

impl TosState {
    pub fn new() -> Self {
        // Initialize module registries
        let mut module_registry = ModuleRegistry::new();
        module_registry.set_default_paths();
        
        let mut app_model_registry = modules::app_model::AppModelRegistry::new();
        app_model_registry.register_builtin_models();
        
        let mut sector_type_registry = modules::sector_type::SectorTypeRegistry::new();
        sector_type_registry.register_builtin_types();
        
        // Initialize marketplace
        let marketplace = marketplace::Marketplace::new();
        if let Err(e) = marketplace.initialize() {
            tracing::warn!("Failed to initialize marketplace: {}", e);
        }
        
        // Try to scan and load modules from default paths
        if let Ok(loaded) = module_registry.scan_and_load() {
            tracing::info!("Loaded {} modules from filesystem", loaded.len());
        }
        let first_sector = Sector {
            id: uuid::Uuid::new_v4(),
            name: "Alpha Sector".to_string(),
            color: "#ff9900".to_string(),
            hubs: vec![CommandHub {
                id: uuid::Uuid::new_v4(),
                mode: CommandHubMode::Command,
                prompt: String::new(),
                applications: vec![Application {
                    id: uuid::Uuid::new_v4(),
                    title: "Main Terminal".to_string(),
                    app_class: "tos.terminal".to_string(),
                    is_minimized: false,
                }],
                active_app_index: Some(0),
                terminal_output: Vec::new(),
                confirmation_required: None,
            }],
            active_hub_index: 0,
            host: "LOCAL".to_string(),
            connection_type: ConnectionType::Local,
            participants: vec![Participant { name: "Host".to_string(), color: "#ffcc00".to_string(), role: "Co-owner".to_string() }],
            portal_active: false,
            portal_url: None,
        };

        let second_sector = Sector {
            id: uuid::Uuid::new_v4(),
            name: "Science Labs".to_string(),
            color: "#9999cc".to_string(),
            hubs: vec![CommandHub {
                id: uuid::Uuid::new_v4(),
                mode: CommandHubMode::Activity,
                prompt: String::new(),
                applications: vec![
                    Application {
                        id: uuid::Uuid::new_v4(),
                        title: "Sensor Array".to_string(),
                        app_class: "labs.sensors".to_string(),
                        is_minimized: false,
                    },
                    Application {
                        id: uuid::Uuid::new_v4(),
                        title: "Stellar Cartography".to_string(),
                        app_class: "labs.astro".to_string(),
                        is_minimized: false,
                    }
                ],
                active_app_index: Some(0),
                terminal_output: Vec::new(),
                confirmation_required: None,
            }],
            active_hub_index: 0,
            host: "LAB-SRV-01".to_string(),
            connection_type: ConnectionType::TOSNative,
            participants: vec![
                Participant { name: "Commander".to_string(), color: "#ffcc00".to_string(), role: "Co-owner".to_string() },
                Participant { name: "Ensign Kim".to_string(), color: "#99ccff".to_string(), role: "Operator".to_string() },
                Participant { name: "Seven".to_string(), color: "#cc99ff".to_string(), role: "Viewer".to_string() },
            ],
            portal_active: false,
            portal_url: None,
        };

        let third_sector = Sector {
            id: uuid::Uuid::new_v4(),
            name: "Observation Hub".to_string(),
            color: "#cc6666".to_string(),
            hubs: vec![CommandHub {
                id: uuid::Uuid::new_v4(),
                mode: CommandHubMode::Command,
                prompt: String::new(),
                applications: vec![Application {
                    id: uuid::Uuid::new_v4(),
                    title: "Remote Desktop".to_string(),
                    app_class: "tos.remote_desktop".to_string(),
                    is_minimized: false,
                }],
                active_app_index: Some(0),
                terminal_output: Vec::new(),
                confirmation_required: None,
            }],
            active_hub_index: 0,
            host: "OBS-NODE-04".to_string(),
            connection_type: ConnectionType::HTTP,
            participants: Vec::new(),
            portal_active: false,
            portal_url: None,
        };

        let initial_viewport = Viewport {
            id: uuid::Uuid::new_v4(),
            sector_index: 0,
            hub_index: 0,
            current_level: HierarchyLevel::GlobalOverview,
            active_app_index: None,
            bezel_expanded: false,
        };

        let state = Self {
            current_level: HierarchyLevel::GlobalOverview,
            sectors: vec![first_sector, second_sector, third_sector],
            viewports: vec![initial_viewport],
            active_viewport_index: 0,
            escape_count: 0,
            fps: 60.0,
            performance_alert: false,
            modules: Vec::new(),
            portal_security_bypass: false,
            approval_requested_sector: None,
            module_registry,
            app_model_registry,
            sector_type_registry,
            marketplace,
            #[cfg(feature = "accessibility")]
            accessibility: None,
            #[cfg(feature = "live-feed")]
            live_feed: None,
            // Phase 11: Initialize new components
            minimap: MiniMap::new(),
            tactical_reset: TacticalReset::new(),
            voice: VoiceCommandProcessor::new(),
            shell_api: ShellApi::new(),
            security: SecurityManager::new(),
        };
        
        // Initialize all loaded modules
        // Note: Module initialization happens after state construction
        // to avoid borrow checker issues with self-referential structs
        let module_names: Vec<String> = state.module_registry.module_names();
        for name in &module_names {
            tracing::info!("Module loaded: {}", name);
        }
        
        state
    }

    pub fn tactical_reset(&mut self) {
        self.current_level = HierarchyLevel::GlobalOverview;
        for viewport in &mut self.viewports {
            viewport.current_level = HierarchyLevel::GlobalOverview;
        }
        self.escape_count = 0;
        
        // Announce reset to accessibility system
        #[cfg(feature = "accessibility")]
        if let Some(ref accessibility) = self.accessibility {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                let accessibility = accessibility.clone();
                handle.spawn(async move {
                    use accessibility::AccessibilityAnnouncement;
                    let _ = accessibility.announce(AccessibilityAnnouncement::Navigation {
                        from_level: "Any".to_string(),
                        to_level: "Global Overview".to_string(),
                        description: "Tactical reset activated".to_string(),
                    }).await;
                });
            }
        }
    }
    
    /// Enable hot-reloading for modules
    pub fn enable_module_hot_reload(&mut self) -> Result<(), String> {
        self.module_registry.enable_hot_reload()
            .map_err(|e| e.to_string())
    }
    
    /// Process file system events for module hot-reload
    pub fn process_module_fs_events(&mut self) {
        // Collect events first to avoid borrow checker issues
        let events: Vec<_> = if let Some(ref receiver) = self.module_registry.event_receiver {
            std::iter::from_fn(|| receiver.try_recv().ok())
                .filter_map(|res| res.ok())
                .collect()
        } else {
            Vec::new()
        };
        
        // Process events
        for event in events {
            match event.kind {
                notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                    for path in &event.paths {
                        if let Some(name) = self.find_module_by_path(path) {
                            tracing::info!("Module changed, reloading: {}", name);
                            let _ = self.reload_module(&name);
                        }
                    }
                }
                notify::EventKind::Remove(_) => {
                    for path in &event.paths {
                        if let Some(name) = self.find_module_by_path(path) {
                            tracing::info!("Module removed: {}", name);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Find a module by its path
    fn find_module_by_path(&self, path: &std::path::Path) -> Option<String> {
        for (name, info) in &self.module_registry.modules {
            if info.path == path || info.path.starts_with(path) {
                return Some(name.clone());
            }
        }
        None
    }
    
    /// Reload a specific module
    fn reload_module(&mut self, name: &str) -> Result<(), String> {
        // Get the module info
        let info = self.module_registry.modules.get_mut(name)
            .ok_or_else(|| format!("Module {} not found", name))?;
        
        // Mark as reloading
        info.state = ModuleState::Reloading;
        
        // Reload manifest
        let manifest_path = info.path.join("module.toml");
        let new_manifest = ModuleManifest::from_toml_file(&manifest_path)
            .map_err(|e| format!("Failed to reload manifest: {}", e))?;
        
        // Update info
        info.manifest = new_manifest;
        info.state = ModuleState::Active;
        info.error = None;
        
        tracing::info!("Module reloaded: {}", name);
        Ok(())
    }
    
    /// Get list of loaded modules
    pub fn list_modules(&self) -> Vec<String> {
        self.module_registry.module_names()
    }
    
    /// Get module count
    pub fn module_count(&self) -> usize {
        self.module_registry.len()
    }
    
    /// Check if a module is loaded
    pub fn is_module_loaded(&self, name: &str) -> bool {
        self.module_registry.is_loaded(name)
    }

    pub fn toggle_bezel(&mut self) {
        self.viewports[self.active_viewport_index].bezel_expanded = !self.viewports[self.active_viewport_index].bezel_expanded;
    }

    pub fn toggle_portal(&mut self) {
        let viewport = &self.viewports[self.active_viewport_index];
        let sector_id = self.sectors[viewport.sector_index].id;
        
        if self.sectors[viewport.sector_index].portal_active {
             let sector = &mut self.sectors[viewport.sector_index];
             sector.portal_active = false;
             sector.portal_url = None;
             return;
        }

        if self.portal_security_bypass {
            self.activate_portal_inner(viewport.sector_index);
        } else {
            self.approval_requested_sector = Some(sector_id);
        }
    }

    fn activate_portal_inner(&mut self, sector_index: usize) {
        let sector = &mut self.sectors[sector_index];
        sector.portal_active = true;
        sector.portal_url = Some(format!("https://tos.grid/portal/{}", &sector.id.to_string()[..8]));
        self.approval_requested_sector = None;
    }

    pub fn approve_portal(&mut self) {
        if let Some(id) = self.approval_requested_sector {
            if let Some(idx) = self.sectors.iter().position(|s| s.id == id) {
                self.activate_portal_inner(idx);
            }
        }
    }

    pub fn deny_portal(&mut self) {
        self.approval_requested_sector = None;
    }

    pub fn is_portal_approval_pending(&self) -> bool {
        let viewport = &self.viewports[self.active_viewport_index];
        self.approval_requested_sector == Some(self.sectors[viewport.sector_index].id)
    }

    pub fn get_approval_requested_sector_name(&self) -> Option<String> {
        self.approval_requested_sector.and_then(|id| {
            self.sectors.iter().find(|s| s.id == id).map(|s| s.name.clone())
        })
    }

    pub fn zoom_in(&mut self) {
        let viewport = &mut self.viewports[self.active_viewport_index];
        match viewport.current_level {
            HierarchyLevel::GlobalOverview => {
                viewport.current_level = HierarchyLevel::CommandHub;
                self.current_level = HierarchyLevel::CommandHub;
            }
            HierarchyLevel::CommandHub => {
                let sector = &self.sectors[viewport.sector_index];
                let hub = &sector.hubs[viewport.hub_index];
                if !hub.applications.is_empty() {
                    viewport.current_level = HierarchyLevel::ApplicationFocus;
                    viewport.active_app_index = Some(0);
                    self.current_level = HierarchyLevel::ApplicationFocus;
                }
            }
            HierarchyLevel::ApplicationFocus => {
                viewport.current_level = HierarchyLevel::DetailInspector;
                self.current_level = HierarchyLevel::DetailInspector;
            }
            HierarchyLevel::DetailInspector => {
                viewport.current_level = HierarchyLevel::BufferInspector;
                self.current_level = HierarchyLevel::BufferInspector;
            }
            _ => {}
        }
    }

    pub fn zoom_out(&mut self) {
        let viewport = &mut self.viewports[self.active_viewport_index];
        match viewport.current_level {
            HierarchyLevel::GlobalOverview => {}
            HierarchyLevel::CommandHub => {
                viewport.current_level = HierarchyLevel::GlobalOverview;
                self.current_level = HierarchyLevel::GlobalOverview;
            }
            HierarchyLevel::ApplicationFocus | HierarchyLevel::SplitView => {
                viewport.current_level = HierarchyLevel::CommandHub;
                self.current_level = HierarchyLevel::CommandHub;
            }
            HierarchyLevel::DetailInspector => {
                viewport.current_level = HierarchyLevel::ApplicationFocus;
                self.current_level = HierarchyLevel::ApplicationFocus;
            }
            HierarchyLevel::BufferInspector => {
                viewport.current_level = HierarchyLevel::DetailInspector;
                self.current_level = HierarchyLevel::DetailInspector;
            }
        }
    }

    pub fn toggle_mode(&mut self, mode: CommandHubMode) {
        let viewport = &self.viewports[self.active_viewport_index];
        if viewport.current_level == HierarchyLevel::CommandHub {
            let sector = &mut self.sectors[viewport.sector_index];
            let hub = &mut sector.hubs[viewport.hub_index];
            hub.mode = mode;
        }
    }

    pub fn set_prompt(&mut self, text: String) {
        let viewport = &self.viewports[self.active_viewport_index];
        if viewport.current_level == HierarchyLevel::CommandHub {
            let sector = &mut self.sectors[viewport.sector_index];
            let hub = &mut sector.hubs[viewport.hub_index];
            hub.prompt = text;
        }
    }

    pub fn stage_command(&mut self, cmd: String) {
        let viewport = &self.viewports[self.active_viewport_index];
        let sector = &mut self.sectors[viewport.sector_index];
        let hub = &mut sector.hubs[viewport.hub_index];
        hub.prompt = cmd;
    }

    pub fn select_sector(&mut self, index: usize) {
        if index < self.sectors.len() {
            self.viewports[self.active_viewport_index].sector_index = index;
            self.viewports[self.active_viewport_index].hub_index = self.sectors[index].active_hub_index;
            self.viewports[self.active_viewport_index].current_level = HierarchyLevel::CommandHub;
            self.current_level = HierarchyLevel::CommandHub;
        }
    }

    pub fn add_sector(&mut self, sector: Sector) {
        self.sectors.push(sector);
    }

    pub fn focus_app_by_id(&mut self, app_id: uuid::Uuid) {
        let viewport_idx = self.active_viewport_index;
        let sector_idx = self.viewports[viewport_idx].sector_index;
        let hub_idx = self.viewports[viewport_idx].hub_index;
        let sector = &mut self.sectors[sector_idx];
        let hub = &mut sector.hubs[hub_idx];

        if let Some(pos) = hub.applications.iter().position(|a| a.id == app_id) {
            hub.active_app_index = Some(pos);
            let viewport = &mut self.viewports[viewport_idx];
            viewport.active_app_index = Some(pos);
            viewport.current_level = HierarchyLevel::ApplicationFocus;
            self.current_level = HierarchyLevel::ApplicationFocus;
        }
    }

    pub fn add_participant(&mut self, sector_index: usize, name: String, color: String, role: String) {
        if let Some(sector) = self.sectors.get_mut(sector_index) {
            sector.participants.push(Participant { name, color, role });
        }
    }

    pub fn update_performance_metrics(&mut self, current_fps: f32) {
        self.fps = current_fps;
        // Trigger alert if FPS is sustained below 30
        if self.fps < 30.0 {
            self.performance_alert = true;
        } else if self.fps > 55.0 {
            self.performance_alert = false;
        }
    }

    pub fn handle_semantic_event(&mut self, event: SemanticEvent) {
        // Broadcast to live feed if enabled
        #[cfg(feature = "live-feed")]
        {
            let live_feed_clone = self.live_feed.clone();
            if let Some(live_feed) = live_feed_clone {
                let rt = tokio::runtime::Handle::try_current();
                if let Ok(handle) = rt {
                    let event_name = format!("{:?}", event);
                    handle.spawn(async move {
                        let _ = live_feed.broadcast_interaction("semantic", &event_name).await;
                    });
                }
            }
        }
        
        // Phase 11: Handle voice activation
        if let SemanticEvent::VoiceCommandStart = event {
            self.voice.simulate_wake_word();
            return;
        }
        
        match event {
            SemanticEvent::ZoomIn => self.zoom_in(),
            SemanticEvent::ZoomOut => self.zoom_out(),
            SemanticEvent::TacticalReset => {
                // Phase 11: Use enhanced tactical reset
                let mut reset = std::mem::take(&mut self.tactical_reset);
                let _ = reset.initiate_sector_reset(self);
                self.tactical_reset = reset;
            }
            SemanticEvent::ToggleBezel => self.toggle_bezel(),
            SemanticEvent::ModeCommand => self.toggle_mode(CommandHubMode::Command),
            SemanticEvent::ModeDirectory => self.toggle_mode(CommandHubMode::Directory),
            SemanticEvent::ModeActivity => self.toggle_mode(CommandHubMode::Activity),
            SemanticEvent::CycleMode => {
                let viewport = &self.viewports[0];
                let current_mode = self.sectors[viewport.sector_index].hubs[viewport.hub_index].mode;
                let next_mode = match current_mode {
                    CommandHubMode::Command => CommandHubMode::Directory,
                    CommandHubMode::Directory => CommandHubMode::Activity,
                    CommandHubMode::Activity => CommandHubMode::Command,
                };
                self.toggle_mode(next_mode);
            }
            SemanticEvent::OpenGlobalOverview => {
                self.current_level = HierarchyLevel::GlobalOverview;
                for v in &mut self.viewports {
                    v.current_level = HierarchyLevel::GlobalOverview;
                }
            }
            _ => {
                // Placeholder for other events
                tracing::info!("Received semantic event: {:?}", event);
            }
        }
        
        // Announce to accessibility system
        #[cfg(feature = "accessibility")]
        self.announce_event(&event);
    }
    
    /// Initialize accessibility system
    #[cfg(feature = "accessibility")]
    pub async fn init_accessibility(&mut self, config: accessibility::AccessibilityConfig) -> Result<(), accessibility::AccessibilityError> {
        let manager = accessibility::AccessibilityManager::new(config).await?;
        self.accessibility = Some(manager);
        tracing::info!("Accessibility system initialized");
        Ok(())
    }
    
    /// Announce semantic event to accessibility system
    #[cfg(feature = "accessibility")]
    fn announce_event(&self, event: &SemanticEvent) {
        if let Some(ref accessibility) = self.accessibility {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                let accessibility = accessibility.clone();
                let announcement = self.event_to_announcement(event);
                handle.spawn(async move {
                    let _ = accessibility.announce(announcement).await;
                });
            }
        }
    }
    
    /// Convert semantic event to accessibility announcement
    #[cfg(feature = "accessibility")]
    fn event_to_announcement(&self, event: &SemanticEvent) -> accessibility::AccessibilityAnnouncement {
        use accessibility::AccessibilityAnnouncement;
        
        match event {
            SemanticEvent::ZoomIn => AccessibilityAnnouncement::Navigation {
                from_level: format!("{:?}", self.current_level),
                to_level: "Deeper".to_string(),
                description: "Zooming in".to_string(),
            },
            SemanticEvent::ZoomOut => AccessibilityAnnouncement::Navigation {
                from_level: format!("{:?}", self.current_level),
                to_level: "Higher".to_string(),
                description: "Zooming out".to_string(),
            },
            SemanticEvent::TacticalReset => AccessibilityAnnouncement::Action {
                action: "Tactical Reset".to_string(),
                result: "Returned to Global Overview".to_string(),
            },
            SemanticEvent::ToggleBezel => AccessibilityAnnouncement::Action {
                action: "Toggle Bezel".to_string(),
                result: "Bezel state changed".to_string(),
            },
            SemanticEvent::ModeCommand => AccessibilityAnnouncement::Status {
                component: "Command Hub".to_string(),
                state: "Command Mode".to_string(),
            },
            SemanticEvent::ModeDirectory => AccessibilityAnnouncement::Status {
                component: "Command Hub".to_string(),
                state: "Directory Mode".to_string(),
            },
            SemanticEvent::ModeActivity => AccessibilityAnnouncement::Status {
                component: "Command Hub".to_string(),
                state: "Activity Mode".to_string(),
            },
            _ => AccessibilityAnnouncement::Action {
                action: format!("{:?}", event),
                result: "Executed".to_string(),
            },
        }
    }
    
    /// Initialize live feed server
    #[cfg(feature = "live-feed")]
    pub async fn init_live_feed(&mut self, config: system::live_feed::LiveFeedConfig) -> Result<(), Box<dyn std::error::Error>> {
        let server = system::live_feed::LiveFeedServer::new(config);
        server.start().await?;
        self.live_feed = Some(server);
        tracing::info!("Live feed server initialized");
        Ok(())
    }
    
    /// Start test recording on live feed
    #[cfg(feature = "live-feed")]
    pub async fn start_test_recording(&self, test_name: &str) {
        if let Some(ref live_feed) = self.live_feed {
            let sender = live_feed.command_sender();
            let _ = sender.send(system::live_feed::FeedCommand::StartRecording(test_name.to_string())).await;
        }
    }
    
    /// Stop test recording
    #[cfg(feature = "live-feed")]
    pub async fn stop_test_recording(&self) {
        if let Some(ref live_feed) = self.live_feed {
            let sender = live_feed.command_sender();
            let _ = sender.send(system::live_feed::FeedCommand::StopRecording).await;
        }
    }

    /// Phase 11: Toggle mini-map activation
    pub fn toggle_minimap(&mut self) {
        self.minimap.toggle();
    }

    /// Phase 11: Process voice text command
    pub fn process_voice_command(&mut self, text: &str) -> Option<system::voice::VoiceCommand> {
        if let Some(cmd) = self.voice.process_text(text) {
            let event = cmd.event.clone();
            self.voice.execute_command(cmd.clone());
            self.handle_semantic_event(event);
            Some(cmd)
        } else {
            None
        }
    }

    /// Phase 11: Check if command is dangerous
    pub fn check_command_security(&self, command: &str) -> Option<(system::security::RiskLevel, String)> {
        self.security.check_command(command)
            .map(|(risk, pattern)| (risk, pattern.message.clone()))
    }

    /// Phase 11: Start security confirmation for command
    pub fn start_security_confirmation(&mut self, command: &str) -> Option<uuid::Uuid> {
        let viewport = &self.viewports[self.active_viewport_index];
        let sector_id = self.sectors[viewport.sector_index].id;
        let user = "current_user".to_string(); // Would get from auth system
        
        self.security.start_confirmation(command, &user, sector_id)
            .map(|session| session.id)
    }

    pub fn render_performance_overlay(&self) -> String {
        ui::render::render_performance_overlay(self.fps, self.performance_alert)
    }

    pub fn render_current_view(&self) -> String {
        let mut html = if self.viewports.len() > 1 {
            self.render_split_view()
        } else {
            let viewport = &self.viewports[0];
            self.render_viewport(viewport)
        };

        if self.performance_alert {
            html.push_str(&self.render_performance_overlay());
        }
        html
    }

    pub fn render_viewport(&self, viewport: &Viewport) -> String {
        use ui::render::ViewRenderer;

        let (mut mode_l1, mut mode_l2, mut mode_l3) = match viewport.current_level {
            HierarchyLevel::GlobalOverview => (RenderMode::Full, RenderMode::Static, RenderMode::Static),
            HierarchyLevel::CommandHub => (RenderMode::Throttled, RenderMode::Full, RenderMode::Static),
            HierarchyLevel::ApplicationFocus | HierarchyLevel::DetailInspector | HierarchyLevel::BufferInspector => 
                (RenderMode::Static, RenderMode::Throttled, RenderMode::Full),
            _ => (RenderMode::Static, RenderMode::Static, RenderMode::Static),
        };

        // Viewport-level throttling: background viewports in split view are penalized
        let is_focused = self.viewports[self.active_viewport_index].id == viewport.id;
        if !is_focused && self.current_level == HierarchyLevel::SplitView {
            mode_l1 = mode_l1.throttle();
            mode_l2 = mode_l2.throttle();
            mode_l3 = mode_l3.throttle();
        }

        match viewport.current_level {
            HierarchyLevel::GlobalOverview => ui::render::global::GlobalRenderer.render(self, viewport, mode_l1),
            HierarchyLevel::CommandHub => ui::render::hub::HubRenderer.render(self, viewport, mode_l2),
            HierarchyLevel::ApplicationFocus => {
                let sector = &self.sectors[viewport.sector_index];
                if sector.connection_type == ConnectionType::HTTP {
                    ui::render::remote::RemoteDesktopRenderer.render(self, viewport, mode_l3)
                } else {
                    ui::render::app::AppRenderer.render(self, viewport, mode_l3)
                }
            },
            HierarchyLevel::DetailInspector => ui::render::inspector::DetailInspectorRenderer.render(self, viewport, mode_l3),
            HierarchyLevel::BufferInspector => ui::render::inspector::BufferInspectorRenderer.render(self, viewport, mode_l3),
            HierarchyLevel::SplitView => self.render_split_view(),
        }
    }

    fn render_split_view(&self) -> String {
        let mut html = String::from(r#"<div class="split-viewport-grid">"#);
        for viewport in &self.viewports {
            html.push_str(&format!(
                r#"<div class="viewport-cell">{}</div>"#,
                self.render_viewport(viewport)
            ));
        }
        html.push_str("</div>");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = TosState::new();
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
        assert_eq!(state.viewports.len(), 1);
        assert_eq!(state.sectors.len(), 3);
    }

    #[test]
    fn test_zoom_transitions() {
        let mut state = TosState::new();
        
        // Zoom into Hub
        state.zoom_in();
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
        assert_eq!(state.viewports[0].current_level, HierarchyLevel::CommandHub);

        // Zoom into App
        state.zoom_in();
        assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
        assert_eq!(state.viewports[0].current_level, HierarchyLevel::ApplicationFocus);

        // Zoom into Detail
        state.zoom_in();
        assert_eq!(state.current_level, HierarchyLevel::DetailInspector);

        // Zoom into Buffer
        state.zoom_in();
        assert_eq!(state.current_level, HierarchyLevel::BufferInspector);

        // Zoom back out to Detail
        state.zoom_out();
        assert_eq!(state.current_level, HierarchyLevel::DetailInspector);

        // Zoom back out to App
        state.zoom_out();
        assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    }

    #[test]
    fn test_tactical_reset() {
        let mut state = TosState::new();
        state.zoom_in();
        state.zoom_in();
        assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
        
        state.tactical_reset();
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
        assert_eq!(state.viewports[0].current_level, HierarchyLevel::GlobalOverview);
    }

    #[test]
    fn test_inspector_rendering() {
        let mut state = TosState::new();
        state.zoom_in(); // Hub
        state.zoom_in(); // Focus
        
        // Detail Inspector
        state.zoom_in();
        let html = state.render_current_view();
        assert!(html.contains("NODE INSPECTOR"));
        assert!(html.contains("UPTIME"));
        assert!(html.contains("tos.terminal")); // Check class name

        // Buffer Inspector
        state.zoom_in();
        let html = state.render_current_view();
        assert!(html.contains("BUFFER HEX DUMP"));
        assert!(html.contains("4c 43 41 52 53")); // "LCARS" in hex
    }

    #[test]
    fn test_bezel_toggling() {
        let mut state = TosState::new();
        state.zoom_in(); // Hub
        state.zoom_in(); // Focus
        
        assert_eq!(state.viewports[0].bezel_expanded, false);
        state.toggle_bezel();
        assert_eq!(state.viewports[0].bezel_expanded, true);
        
        let html = state.render_current_view();
        assert!(html.contains("tactical-bezel expanded"));
        assert!(html.contains("PRIORITY"));
    }

    #[test]
    fn test_semantic_events() {
        let mut state = TosState::new();
        
        // Test Zoom In
        state.handle_semantic_event(SemanticEvent::ZoomIn);
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
        
        // Test Zoom Out
        state.handle_semantic_event(SemanticEvent::ZoomOut);
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
        
        // Test Mode Switching via CycleMode
        state.handle_semantic_event(SemanticEvent::ZoomIn);
        state.handle_semantic_event(SemanticEvent::CycleMode);
        let viewport = &state.viewports[0];
        assert_eq!(state.sectors[viewport.sector_index].hubs[viewport.hub_index].mode, CommandHubMode::Directory);
        
        // Test Tactical Reset
        state.handle_semantic_event(SemanticEvent::TacticalReset);
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    }
    #[test]
    fn test_render_modes() {
        let mut state = TosState::new();
        
        // Level 1: Global Overview should be Full
        let html = state.render_current_view();
        assert!(html.contains("mode-Full"));
        
        // Level 2: Command Hub should be Full
        state.zoom_in();
        let html = state.render_current_view();
        assert!(html.contains("render-Full"));
        
        // Level 3: Application should be Full
        state.zoom_in();
        let html = state.render_current_view();
        assert!(html.contains("render-Full"));
    }

    #[test]
    fn test_module_integration() {
        let mut state = TosState::new();
        state.modules.push(Box::new(EngineeringModule { power_distribution: [100, 80, 50] }));
        
        state.zoom_in(); // Hub
        state.zoom_in(); // Focus
        
        let html = state.render_current_view();
        assert!(html.contains("PROPULSION: 100%"));
        assert!(html.contains("SHIELDS: 80%"));
    }
}
