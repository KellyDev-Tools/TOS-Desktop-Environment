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
use system::status::StatusBar;
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
    pub status: StatusBar,
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
            status: StatusBar::new(),
            current_morph_phase: MorphPhase::Static,
        }
    }

    pub fn tick(&mut self) {
        let titles = self.surfaces.get_all_surface_titles();
        for widget in &mut self.dashboard.widgets {
            widget.update();
            if let Some(pm) = widget.as_any_mut().downcast_mut::<ui::dashboard::ProcessManagerWidget>() {
                pm.processes = titles.clone();
            }
        }
        self.status.tick();
    }

    pub fn start_zoom_morph(&mut self, entering: bool) {
        self.current_morph_phase = if entering { MorphPhase::Entering } else { MorphPhase::Exiting };
    }

    pub fn finish_morph(&mut self) {
        self.current_morph_phase = MorphPhase::Static;
    }

    pub fn generate_viewport_html(&self) -> String {
        let mut html = String::new();

        // 1. Status Bar (Global Top Layer)
        html.push_str(&self.status.render_html(self.navigator.current_level, self.navigator.active_sector_index));

        // 2. Dashboard if visible
        use navigation::zoom::ZoomLevel;
        if matches!(self.navigator.current_level, ZoomLevel::Level1Root | ZoomLevel::Level2Sector) {
            html.push_str("<div class='dashboard-layer'>");
            html.push_str(&self.dashboard.render_all_html());
            html.push_str("</div>");
        }

        // 3. Surfaces layer (Spatial Grid)
        let primary_id = if let (Some(sector), Some(app)) = (self.navigator.active_sector_index, self.navigator.active_app_index) {
            // This is a bit of a hack since navigator uses indices and surfaces uses IDs
            // In a real impl, those would be unified. For now, let's find the ID.
            self.surfaces.get_surfaces_in_sector(sector).get(app).map(|s| s.id)
        } else {
            None
        };

        let layouts = SpatialMapper::get_layout(
            &self.surfaces, 
            self.navigator.current_level, 
            self.navigator.active_sector_index, 
            primary_id,
            self.navigator.secondary_app_id
        );

        html.push_str("<div class='surfaces-grid'>");
        for layout in layouts {
            let grid_style = format!(
                "grid-column: span {}; grid-row: span {};",
                layout.width, layout.height
            );
            
            html.push_str(&format!(r#"<div class="grid-item" style="{}">"#, grid_style));
            
            let mut decoration = DecorationManager::get_html_frame(
                &layout.surface.title, 
                DecorationStyle::Default, 
                self.current_morph_phase,
                &format!("surface-{}", layout.surface.id)
            );

            // Level 4: Deep detail injection
            if self.navigator.current_level == ZoomLevel::Level4Detail {
                let detail_mock = format!(
                    r#"<div class="lcars-detail-box">
                        <div class="detail-header">NODE INSPECTOR: {}</div>
                        <div class="detail-row"><span>STATUS:</span> <span class="active">ACTIVE</span></div>
                        <div class="detail-row"><span>THREADS:</span> 12</div>
                        <div class="detail-row"><span>MEMORY:</span> 256MB</div>
                        <div class="detail-row"><span>UPTIME:</span> {}s</div>
                        <div class="detail-chart">
                            <div class="bar" style="height: 60%;"></div>
                            <div class="bar" style="height: 40%;"></div>
                            <div class="bar" style="height: 80%;"></div>
                            <div class="bar" style="height: 20%;"></div>
                        </div>
                    </div>"#,
                    layout.surface.id, self.status.uptime_secs
                );
                decoration = decoration.replace("<!-- Surface content injected here -->", &detail_mock);
            }

            html.push_str(&decoration);
            html.push_str("</div>");
        }
        html.push_str("</div>");

        html
    }
}
