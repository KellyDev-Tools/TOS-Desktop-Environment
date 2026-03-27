pub mod navigation;
pub mod ui;
pub mod system;
pub mod compositor;

#[cfg(feature = "dev-monitor")]
pub mod dev_monitor;

use navigation::zoom::SpatialNavigator;
use ui::dashboard::Dashboard;
use ui::decorations::{DecorationManager, DecorationStyle, MorphPhase};
use system::notifications::NotificationManager;
use system::files::VirtualFileSystem;
use system::audio::AudioFeedback;
use system::shell::{ShellIntegrator, ShellCommand};
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
        is_red_alert: bool,
    },
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub audio_enabled: bool,
    pub chirps_enabled: bool,
    pub ambient_enabled: bool,
    pub high_contrast: bool,
    pub debug_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            audio_enabled: true,
            chirps_enabled: true,
            ambient_enabled: true,
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
    pub is_red_alert: bool,
    pub viewport_manager: navigation::viewport::ViewportManager,
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
            is_red_alert: false,
            viewport_manager: navigation::viewport::ViewportManager::new(),
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
        self.surfaces.update_telemetry();
        self.status.tick();
        self.audio.tick();

        // Auto-trigger red alert if there are critical notifications
        self.is_red_alert = self.notifications.queue.iter().any(|n| matches!(n.priority, crate::system::notifications::Priority::Critical));
    }

    pub fn handle_shell_output(&mut self, data: &str) {
        let commands = self.shell.parse_stdout(data);
        for cmd in commands {
            match cmd {
                ShellCommand::Zoom(level) => {
                    // Sync internal navigator
                    let target = match level {
                        1 => navigation::zoom::ZoomLevel::Level1Root,
                        2 => navigation::zoom::ZoomLevel::Level2Sector,
                        3 => navigation::zoom::ZoomLevel::Level3Focus,
                        4 => navigation::zoom::ZoomLevel::Level4Detail,
                        _ => self.navigator.current_level,
                    };
                    self.navigator.current_level = target;
                    println!("[Brain] Shell synced zoom to level {}", level);
                }
                ShellCommand::ChangeDir(path) => {
                    self.files.current_path = path;
                    println!("[Brain] Shell synced dir to {}", self.files.current_path);
                }
                ShellCommand::SetLayout(_) => {}
            }
        }
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
            let mut search_layouts = Vec::new();
            
            // 1. Surface matches
            let matches = self.surfaces.find_surfaces(query);
            for (i, surface) in matches.into_iter().enumerate() {
                search_layouts.push(compositor::SurfaceLayout {
                    surface,
                    grid_x: (i % 3) as u16,
                    grid_y: (i / 3) as u16,
                    width: 1,
                    height: 1,
                });
            }

            // 2. File matches (mocked as surfaces for rendering)
            let file_matches = self.files.search(query);
            let offset = search_layouts.len();
            for (i, (path, node)) in file_matches.into_iter().enumerate() {
                let idx = offset + i;
                search_layouts.push(compositor::SurfaceLayout {
                    surface: compositor::TosSurface {
                        id: 5000 + idx as u32,
                        title: format!("FILE: {}", node.name),
                        app_class: "FileSearch".to_string(),
                        role: compositor::SurfaceRole::Toplevel,
                        sector_id: None,
                        history: vec![format!("Found at: {}", path)],
                        cpu_usage: 0,
                        mem_usage: 0,
                    },
                    grid_x: (idx % 3) as u16,
                    grid_y: (idx / 3) as u16,
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
                 let is_file = layout.surface.app_class == "FileSearch";
                 let (color, location) = if is_file {
                     ("var(--lcars-orange)".to_string(), layout.surface.history[0].clone())
                 } else {
                     let sid = layout.surface.sector_id.unwrap_or(0);
                     (self.navigator.sectors[sid].color.clone(), format!("SECTOR: {}", self.navigator.sectors[sid].name))
                 };

                 let search_item_html = format!(
                    r#"<div class="lcars-search-result" onclick="sendCommand('terminal:zoom 2'); sendCommand('zoom:3:{}')" style="border-left-color: {}">
                        <div class="result-title">{}</div>
                        <div class="result-sector">{}</div>
                    </div>"#,
                    layout.surface.id, color, layout.surface.title, location
                );
                DecorationManager::get_html_frame(
                    &layout.surface.title, 
                    DecorationStyle::Default, 
                    self.current_morph_phase,
                    &format!("search-{}", layout.surface.id)
                ).replace("<!-- Surface content injected here -->", &search_item_html)
            } else {
                let mut frame = DecorationManager::get_html_frame(
                    &layout.surface.title, 
                    DecorationStyle::Default, 
                    self.current_morph_phase,
                    &format!("surface-{}", layout.surface.id)
                );
                
                if self.navigator.current_level == ZoomLevel::Level2Sector {
                    let orch_id = format!(r#"<div class="orch-id-label">ID: {}</div>"#, layout.surface.id);
                    frame = frame.replace("<!-- Surface content injected here -->", &orch_id);
                }
                frame
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
                            <div class="detail-row"><span>CPU LOAD:</span> {1}%</div>
                            <div class="detail-row"><span>MEM LOAD:</span> {2}%</div>
                            <div class="detail-row"><span>UPTIME:</span> {3}s</div>
                            <div class="detail-button" onclick="sendCommand('zoom:in')">ACCESS RAW BUFFER</div>
                            <div class="detail-chart">
                                <div class="bar" style="height: {1}%;"></div>
                                <div class="bar" style="height: {2}%;"></div>
                                <div class="bar" style="height: 30%;"></div>
                                <div class="bar" style="height: 10%;"></div>
                            </div>
                        </div>
                        <div class="lcars-detail-box right-panel">
                            <div class="detail-header">NODE HISTORY</div>
                            <div class="history-list">
                                {4}
                            </div>
                        </div>
                    </div>"#,
                    layout.surface.id, layout.surface.cpu_usage, layout.surface.mem_usage, self.status.uptime_secs, history_html
                );
                decoration = decoration.replace("<!-- Surface content injected here -->", &detail_mock);
            } else if self.navigator.current_level == ZoomLevel::Level5Buffer {
                let mut hex_lines = String::new();
                for i in 0..16 {
                    let addr = i * 16;
                    let mut hex = String::new();
                    let mut chars = String::new();
                    for j in 0..16 {
                        let val = (compositor::id_to_noise(layout.surface.id, (i * 16 + j) as u32) % 256) as u8;
                        hex.push_str(&format!("{:02X} ", val));
                        if val >= 32 && val <= 126 {
                            chars.push(val as char);
                        } else {
                            chars.push('.');
                        }
                    }
                    hex_lines.push_str(&format!(
                        r#"<div class="hex-line">
                            <span class="hex-addr">{:08X}</span>
                            <span class="hex-data">{}</span>
                            <span class="hex-chars">{}</span>
                        </div>"#,
                        addr, hex, chars
                    ));
                }

                let buffer_html = format!(
                    r#"<div class="lcars-memory-buffer">
                        <div class="buffer-header">RAW MEMORY BUFFER: {0}</div>
                        <div class="hex-scroll">
                            {1}
                        </div>
                    </div>"#,
                    layout.surface.title, hex_lines
                );
                decoration = decoration.replace("<!-- Surface content injected here -->", &buffer_html);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_environment_initialization() {
        let env = DesktopEnvironment::new(None);
        
        // Verify all components initialized
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level1Root);
        assert_eq!(env.dashboard.widgets.len(), 0);
        assert_eq!(env.notifications.queue.len(), 0);
        assert_eq!(env.files.current_path, "/home/user");
        assert!(env.audio.enabled);
        assert_eq!(env.surfaces.get_all_surface_titles().len(), 0);
        assert_eq!(env.current_morph_phase, MorphPhase::Static);
        assert!(env.search_query.is_none());
        assert!(env.settings.audio_enabled);
        assert!(!env.is_red_alert);
    }

    #[test]
    fn test_tick_increments_state() {
        let mut env = DesktopEnvironment::new(None);
        
        // Initial uptime should be 0
        let initial_uptime = env.status.uptime_secs;
        
        // Tick once
        env.tick();
        
        // Status should have ticked (uptime incremented)
        assert_ne!(env.status.uptime_secs, initial_uptime);
        
        // Audio should have ticked
        // (no easy way to verify without exposing ambient_timer, but at least no panic)
    }

    #[test]
    fn test_tick_updates_widgets() {
        let mut env = DesktopEnvironment::new(None);
        
        // Add a system monitor widget
        env.dashboard.add_widget(Box::new(ui::dashboard::SystemMonitorWidget {
            cpu_usage: 10,
            ram_usage: 20,
        }));
        
        // Create some surfaces
        env.surfaces.create_surface("App1", compositor::SurfaceRole::Toplevel, Some(0));
        env.surfaces.create_surface("App2", compositor::SurfaceRole::Toplevel, Some(1));
        
        // Add process manager widget
        env.dashboard.add_widget(Box::new(ui::dashboard::ProcessManagerWidget {
            processes: vec![],
        }));
        
        // Tick to update widgets
        env.tick();
        
        // Verify widgets were updated (system monitor CPU should have incremented)
        // Note: update() increments cpu by 5
        let widget = &env.dashboard.widgets[0];
        let monitor = widget.as_any().downcast_ref::<ui::dashboard::SystemMonitorWidget>().unwrap();
        assert_eq!(monitor.cpu_usage, 15); // 10 + 5
    }

    #[test]
    fn test_tick_triggers_red_alert() {
        let mut env = DesktopEnvironment::new(None);
        
        // Initially no red alert
        assert!(!env.is_red_alert);
        
        // Add critical notification
        env.notifications.push("CRITICAL", "Core breach!", system::notifications::Priority::Critical);
        
        // Tick to process
        env.tick();
        
        // Should now be in red alert
        assert!(env.is_red_alert);
    }

    #[test]
    fn test_handle_shell_output_parses_osc_zoom() {
        let mut env = DesktopEnvironment::new(None);
        
        // Initially at Level 1
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level1Root);
        
        // Send shell OSC command to zoom to level 2
        env.handle_shell_output("\x1b]1337;ZoomLevel=2\x07");
        
        // Navigator should have synced
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level2Sector);
    }

    #[test]
    fn test_handle_shell_output_syncs_directory() {
        let mut env = DesktopEnvironment::new(None);
        
        // Initial path
        assert_eq!(env.files.current_path, "/home/user");
        
        // Shell sends directory change
        env.handle_shell_output("\x1b]1337;CurrentDir=/tmp\x07");
        
        // VFS should have synced
        assert_eq!(env.files.current_path, "/tmp");
    }

    #[test]
    fn test_handle_shell_output_no_osc() {
        let mut env = DesktopEnvironment::new(None);
        
        let initial_level = env.navigator.current_level;
        let initial_path = env.files.current_path.clone();
        
        // Send plain text (no OSC sequences)
        env.handle_shell_output("just some normal terminal output");
        
        // Nothing should have changed
        assert_eq!(env.navigator.current_level, initial_level);
        assert_eq!(env.files.current_path, initial_path);
    }

    #[test]
    fn test_intelligent_zoom_out_single_window() {
        let mut env = DesktopEnvironment::new(None);
        let _id = env.surfaces.create_surface("OnlyOne", compositor::SurfaceRole::Toplevel, Some(0));
        
        // Navigate to focus
        env.navigator.zoom_in(0); // Sector 0
        env.navigator.zoom_in(0); // Focus first app
        env.navigator.active_app_index = Some(0);
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level3Focus);
        
        // Intelligent zoom out (single window, should go to sector)
        env.intelligent_zoom_out();
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level2Sector);
    }

    #[test]
    fn test_intelligent_zoom_out_multiple_windows() {
        let mut env = DesktopEnvironment::new(None);
        env.surfaces.create_surface("Terminal", compositor::SurfaceRole::Toplevel, Some(0));
        env.surfaces.create_surface("Terminal", compositor::SurfaceRole::Toplevel, Some(0));
        
        // Navigate to focus
        env.navigator.zoom_in(0);
        env.navigator.zoom_in(0);
        env.navigator.active_app_index = Some(0);
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level3Focus);
        
        // Intelligent zoom out (multiple windows, should go to picker)
        env.intelligent_zoom_out();
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level3aPicker);
    }

    #[test]
    fn test_morph_phase_transitions() {
        let mut env = DesktopEnvironment::new(None);
        
        assert_eq!(env.current_morph_phase, MorphPhase::Static);
        
        env.start_zoom_morph(true);
        assert_eq!(env.current_morph_phase, MorphPhase::Entering);
        
        env.finish_morph();
        assert_eq!(env.current_morph_phase, MorphPhase::Static);
        
        env.start_zoom_morph(false);
        assert_eq!(env.current_morph_phase, MorphPhase::Exiting);
        
        env.finish_morph();
        assert_eq!(env.current_morph_phase, MorphPhase::Static);
    }

    #[test]
    fn test_generate_viewport_html_structure() {
        let mut env = DesktopEnvironment::new(None);
        
        let html = env.generate_viewport_html();
        
        // Should contain status bar
        assert!(html.contains("LOC:"));
        
        // Should contain surfaces grid
        assert!(html.contains("surfaces-grid"));
        
        // Should be valid HTML structure (closing tags)
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_generate_viewport_includes_audio_buffer() {
        let mut env = DesktopEnvironment::new(None);
        
        // Queue some audio
        env.audio.play_sound("test_sound");
        env.audio.play_sound("another_sound");
        
        let html = env.generate_viewport_html();
        
        // Audio buffer should be in HTML
        assert!(html.contains("audio-buffer"));
        assert!(html.contains("test_sound"));
        assert!(html.contains("another_sound"));
        
        // Audio queue should be consumed now
        assert_eq!(env.audio.queue.len(), 0);
    }

    #[test]
    fn test_generate_viewport_dashboard_at_root() {
        let mut env = DesktopEnvironment::new(None);
        env.dashboard.add_widget(Box::new(ui::dashboard::ClockWidget));
        
        // At root level, dashboard should be included
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level1Root);
        let html = env.generate_viewport_html();
        assert!(html.contains("dashboard-layer"));
        assert!(html.contains("CLOCK"));
    }

    #[test]
    fn test_generate_viewport_no_dashboard_at_focus() {
        let mut env = DesktopEnvironment::new(None);
        env.dashboard.add_widget(Box::new(ui::dashboard::ClockWidget));
        env.surfaces.create_surface("App", compositor::SurfaceRole::Toplevel, Some(0));
        
        // Navigate to focus
        env.navigator.zoom_in(0);
        env.navigator.zoom_in(0);
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level3Focus);
        
        // Dashboard should not be in HTML at focus level
        let html = env.generate_viewport_html();
        assert!(!html.contains("dashboard-layer"));
    }

    #[test]
    fn test_swap_split_functionality() {
        let mut env = DesktopEnvironment::new(None);
        let s1 = env.surfaces.create_surface("S1", compositor::SurfaceRole::Toplevel, Some(0));
        let s2 = env.surfaces.create_surface("S2", compositor::SurfaceRole::Toplevel, Some(0));
        
        // Navigate to split view
        env.navigator.zoom_in(0);
        env.navigator.zoom_in(0);
        env.navigator.active_app_index = Some(0);
        env.navigator.split_view(s2);
        
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level3Split);
        assert_eq!(env.navigator.secondary_app_id, Some(s2));
        
        // Swap
        let success = env.swap_split();
        assert!(success);
        
        // Primary and secondary should have swapped
        assert_eq!(env.navigator.secondary_app_id, Some(s1));
    }

    #[test]
    fn test_swap_split_fails_when_not_in_split() {
        let mut env = DesktopEnvironment::new(None);
        
        // Not in split view
        assert_eq!(env.navigator.current_level, navigation::zoom::ZoomLevel::Level1Root);
        
        let success = env.swap_split();
        assert!(!success);
    }

    #[test]
    fn test_settings_defaults() {
        let settings = AppSettings::default();
        
        assert!(settings.audio_enabled);
        assert!(settings.chirps_enabled);
        assert!(settings.ambient_enabled);
        assert!(!settings.high_contrast);
        assert!(!settings.debug_mode);
    }
}

