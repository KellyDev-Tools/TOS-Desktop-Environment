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

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub audio_enabled: bool,
    pub chirps_enabled: bool,
    pub high_contrast: bool,
    pub debug_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            audio_enabled: true,
            chirps_enabled: true,
            high_contrast: false,
            debug_mode: false,
        }
    }
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
    pub search_query: Option<String>,
    pub settings: AppSettings,
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
            search_query: None,
            settings: AppSettings::default(),
        }
    }

    pub fn tick(&mut self) {
        let titles = self.surfaces.get_all_surface_titles();
        let audio_on = self.settings.audio_enabled;
        let chirps_on = self.settings.chirps_enabled;

        for widget in &mut self.dashboard.widgets {
            widget.update();
            if let Some(pm) = widget.as_any_mut().downcast_mut::<ui::dashboard::ProcessManagerWidget>() {
                pm.processes = titles.clone();
            }
            if let Some(sw) = widget.as_any_mut().downcast_mut::<ui::dashboard::SettingsWidget>() {
                sw.audio_on = audio_on;
                sw.chirps_on = chirps_on;
            }
        }
        self.status.tick();
    }

    pub fn intelligent_zoom_out(&mut self) {
        let has_multi = if self.navigator.current_level == navigation::zoom::ZoomLevel::Level3Focus {
            if let (Some(sector), Some(app_idx)) = (self.navigator.active_sector_index, self.navigator.active_app_index) {
                if let Some(surface) = self.surfaces.get_surfaces_in_sector(sector).get(app_idx) {
                    self.surfaces.get_surfaces_in_group(&surface.app_class).len() > 1
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        
        self.navigator.zoom_out(has_multi);
    }

    pub fn start_zoom_morph(&mut self, entering: bool) {
        self.current_morph_phase = if entering { MorphPhase::Entering } else { MorphPhase::Exiting };
    }

    pub fn finish_morph(&mut self) {
        self.current_morph_phase = MorphPhase::Static;
    }

    pub fn swap_split(&mut self) -> bool {
        if self.navigator.current_level == navigation::zoom::ZoomLevel::Level3Split {
            let primary_id = if let (Some(sector), Some(app)) = (self.navigator.active_sector_index, self.navigator.active_app_index) {
                self.surfaces.get_surfaces_in_sector(sector).get(app).map(|s| s.id)
            } else {
                None
            };

            if let (Some(p_id), Some(s_id)) = (primary_id, self.navigator.secondary_app_id) {
                // To swap, we need to find the sector and index of s_id
                if let Some(s_surface) = self.surfaces.get_surface(s_id) {
                    if let Some(s_sector) = s_surface.sector_id {
                        let sector_surfaces = self.surfaces.get_surfaces_in_sector(s_sector);
                        if let Some(s_idx) = sector_surfaces.iter().position(|s| s.id == s_id) {
                            self.navigator.active_sector_index = Some(s_sector);
                            self.navigator.active_app_index = Some(s_idx);
                            self.navigator.secondary_app_id = Some(p_id);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn generate_viewport_html(&mut self) -> String {
        let mut html = String::new();

        // Consume audio queue and inject triggers
        let audio_events = self.audio.consume_queue();
        if !audio_events.is_empty() {
            html.push_str("<div id='audio-buffer' style='display:none' data-sounds='");
            html.push_str(&audio_events.join(","));
            html.push_str("'></div>");
        }

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

        let layouts = if self.navigator.current_level == ZoomLevel::Level1Root && self.search_query.is_some() {
            let query = self.search_query.as_ref().unwrap();
            let matches = self.surfaces.find_surfaces(query);
            let mut search_layouts = Vec::new();
            for (i, surface) in matches.into_iter().enumerate() {
                let x = (i % 3) as u16;
                let y = (i / 3) as u16;
                search_layouts.push(compositor::SurfaceLayout {
                    surface,
                    grid_x: x,
                    grid_y: y,
                    width: 1,
                    height: 1,
                });
            }
            search_layouts
        } else {
            SpatialMapper::get_layout(
                &self.surfaces, 
                self.navigator.current_level, 
                self.navigator.active_sector_index, 
                primary_id,
                self.navigator.secondary_app_id,
                self.navigator.sectors.len()
            )
        };

        html.push_str("<div class='surfaces-grid'>");
        for layout in layouts {
            let sector_class = if let Some(sid) = layout.surface.sector_id {
                format!("sector-{}-slide", sid)
            } else {
                "".to_string()
            };

            let morph_type = match self.current_morph_phase {
                MorphPhase::Entering => "morph-entering",
                MorphPhase::Exiting => "morph-exiting",
                MorphPhase::Static => "",
            };

            let grid_style = format!(
                "grid-column: span {}; grid-row: span {};",
                layout.width, layout.height
            );
            
            html.push_str(&format!(r#"<div class="grid-item {} {}" style="{}">"#, sector_class, morph_type, grid_style));
            
            let mut decoration = if self.navigator.current_level == ZoomLevel::Level1Root && self.search_query.is_none() {
                let sector_idx = layout.surface.sector_id.unwrap_or(0);
                let sector_info = &self.navigator.sectors[sector_idx];
                let app_count = self.surfaces.get_surfaces_in_sector(sector_idx).len();
                
                let sector_html = format!(
                    r#"<div class="lcars-sector-card" onclick="sendCommand('zoom:2:{}')" style="border-left-color: {}">
                        <div class="sector-title">{}</div>
                        <div class="sector-stats">{} ACTIVE APPS</div>
                        <div class="sector-mini-grid">
                            <div class="mini-box"></div>
                            <div class="mini-box"></div>
                            <div class="mini-box"></div>
                        </div>
                    </div>"#,
                    sector_idx, sector_info.color, sector_info.name, app_count
                );
                
                DecorationManager::get_html_frame(
                    &sector_info.name, 
                    DecorationStyle::Default, 
                    self.current_morph_phase,
                    &format!("sector-{}", sector_idx)
                ).replace("<!-- Surface content injected here -->", &sector_html)
            } else if self.navigator.current_level == ZoomLevel::Level3aPicker {
                 let group = self.surfaces.get_surfaces_in_group(&layout.surface.app_class);
                 let window_idx = group.iter().position(|s| s.id == layout.surface.id).unwrap_or(0);
                 
                 let picker_html = format!(
                    r#"<div class="lcars-picker-item" onclick="sendCommand('zoom:3:{}')">
                        <div class="picker-label">SELECT WINDOW</div>
                        <div class="picker-title">{}</div>
                    </div>"#,
                    window_idx, layout.surface.title
                 );
                 
                 DecorationManager::get_html_frame(
                    &layout.surface.title, 
                    DecorationStyle::Default, 
                    self.current_morph_phase,
                    &format!("surface-{}", layout.surface.id)
                ).replace("<!-- Surface content injected here -->", &picker_html)
            } else if self.navigator.current_level == ZoomLevel::Level1Root && self.search_query.is_some() {
                 let sector_info = &self.navigator.sectors[layout.surface.sector_id.unwrap_or(0)];
                 let search_item_html = format!(
                    r#"<div class="lcars-search-result" onclick="sendCommand('terminal:zoom 2'); sendCommand('zoom:3:{}')" style="border-left-color: {}">
                        <div class="result-title">{}</div>
                        <div class="result-sector">SECTOR: {}</div>
                    </div>"#,
                    layout.surface.id, sector_info.color, layout.surface.title, sector_info.name
                );
                DecorationManager::get_html_frame(
                    &layout.surface.title, 
                    DecorationStyle::Default, 
                    self.current_morph_phase,
                    &format!("search-{}", layout.surface.id)
                ).replace("<!-- Surface content injected here -->", &search_item_html)
            } else {
                DecorationManager::get_html_frame(
                    &layout.surface.title, 
                    DecorationStyle::Default, 
                    self.current_morph_phase,
                    &format!("surface-{}", layout.surface.id)
                )
            };

            // Level 4: Deep detail injection
            if self.navigator.current_level == ZoomLevel::Level4Detail {
                let mut history_html = String::new();
                for event in &layout.surface.history {
                    history_html.push_str(&format!(r#"<div class="history-item">{}</div>"#, event));
                }

                let detail_mock = format!(
                    r#"<div class="lcars-detail-wrapper">
                        <div class="lcars-detail-box left-panel">
                            <div class="detail-header">NODE INSPECTOR: {0}</div>
                            <div class="detail-row"><span>STATUS:</span> <span class="active">ACTIVE</span></div>
                            <div class="detail-row"><span>THREADS:</span> 12</div>
                            <div class="detail-row"><span>MEMORY:</span> 256MB</div>
                            <div class="detail-row"><span>UPTIME:</span> {1}s</div>
                            <div class="detail-chart">
                                <div class="bar" style="height: 60%;"></div>
                                <div class="bar" style="height: 40%;"></div>
                                <div class="bar" style="height: 80%;"></div>
                                <div class="bar" style="height: 20%;"></div>
                            </div>
                        </div>
                        <div class="lcars-detail-box right-panel">
                            <div class="detail-header">NODE HISTORY</div>
                            <div class="history-list">
                                {2}
                            </div>
                        </div>
                    </div>"#,
                    layout.surface.id, self.status.uptime_secs, history_html
                );
                decoration = decoration.replace("<!-- Surface content injected here -->", &detail_mock);
            } else if layout.surface.title.to_lowercase().contains("file") {
                // Spatial File System injection
                let mut files_html = format!(r#"<div class="lcars-file-browser"><div class="file-path">{}</div><div class="file-grid">"#, self.files.current_path);
                if let Some(entries) = self.files.get_current_entries() {
                    for entry in entries {
                        let class = if entry.is_dir { "file-item dir" } else { "file-item" };
                        let icon = if entry.is_dir { "📁" } else { "📄" };
                        files_html.push_str(&format!(
                            r#"<div class="{}" onclick="sendCommand('terminal:cd {}')">
                                <div class="file-icon">{}</div>
                                <div class="file-name">{}</div>
                            </div>"#,
                            class, entry.name, icon, entry.name
                        ));
                    }
                }
                files_html.push_str("</div></div>");
                decoration = decoration.replace("<!-- Surface content injected here -->", &files_html);
            }

            html.push_str(&decoration);
            html.push_str("</div>");
        }
        html.push_str("</div>");

        html
    }
}
