pub mod navigation;
pub mod ui;
pub mod system;
pub mod compositor;

use navigation::zoom::SpatialNavigator;
use ui::dashboard::Dashboard;
use ui::decorations::{DecorationManager, DecorationStyle, MorphPhase};
use system::notifications::NotificationManager;
use system::files::VirtualFileSystem;
use system::audio::AudioFeedback;
use system::shell::ShellIntegrator;
use compositor::{SurfaceManager, SpatialMapper};
use std::sync::mpsc::Sender;

// Shared command enum — used by both the UI thread and the Brain thread
#[derive(Debug, Clone)]
pub enum UiCommand {
    UpdateDashboard(String), 
    ZoomLevel(u8),
    UpdateViewport { 
        html_content: String,
        zoom_level: u8,
    },
}

pub struct DesktopEnvironment {
    pub navigator: SpatialNavigator,
    pub dashboard: Dashboard,
    pub notifications: NotificationManager,
    pub files: VirtualFileSystem,
    pub audio: AudioFeedback,
    pub shell: ShellIntegrator,
    pub surfaces: SurfaceManager,
    pub current_morph_phase: MorphPhase,
}

impl DesktopEnvironment {
    pub fn new(ui_tx: Option<Sender<UiCommand>>) -> Self {
        Self {
            navigator: SpatialNavigator::new(),
            dashboard: Dashboard::new(),
            notifications: NotificationManager::new(),
            files: VirtualFileSystem::new(),
            audio: AudioFeedback::new(),
            shell: ShellIntegrator::new(ui_tx),
            surfaces: SurfaceManager::new(),
            current_morph_phase: MorphPhase::Static,
        }
    }

    pub fn tick(&mut self) {
        self.dashboard.widgets.iter_mut().for_each(|w| w.update());
    }

    // Call this before navigation to trigger an animation
    pub fn start_zoom_morph(&mut self, entering: bool) {
        self.current_morph_phase = if entering { MorphPhase::Entering } else { MorphPhase::Exiting };
    }

    pub fn finish_morph(&mut self) {
        self.current_morph_phase = MorphPhase::Static;
    }

    pub fn generate_viewport_html(&self) -> String {
        let visible_surfaces = SpatialMapper::get_visible_surfaces(
            &self.surfaces, 
            self.navigator.current_level, 
            self.navigator.active_sector_index, 
            None 
        );

        let mut html = String::new();
        
        use navigation::zoom::ZoomLevel;
        if matches!(self.navigator.current_level, ZoomLevel::Level1Root | ZoomLevel::Level2Sector) {
            html.push_str("<div class='dashboard-layer'>");
            html.push_str(&self.dashboard.render_all_html());
            html.push_str("</div>");
        }

        html.push_str("<div class='surfaces-layer'>");
        for surface in visible_surfaces {
            html.push_str(&DecorationManager::get_html_frame(
                &surface.title, 
                DecorationStyle::Default, 
                self.current_morph_phase,
                &format!("surface-{}", surface.id)
            ));
        }
        html.push_str("</div>");

        html
    }
}
