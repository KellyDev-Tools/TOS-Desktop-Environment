use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandHubMode {
    Command,
    Directory,
    Activity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: uuid::Uuid,
    pub name: String,
    pub color: String,
    pub hubs: Vec<CommandHub>,
    pub active_hub_index: usize,
}

pub mod system;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHub {
    pub id: uuid::Uuid,
    pub mode: CommandHubMode,
    pub prompt: String,
    pub applications: Vec<Application>,
    pub active_app_index: Option<usize>,
    pub terminal_output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: uuid::Uuid,
    pub title: String,
    pub app_class: String,
    pub is_minimized: bool,
}

pub struct TosState {
    pub current_level: HierarchyLevel,
    pub sectors: Vec<Sector>,
    pub viewports: Vec<Viewport>,
    pub active_viewport_index: usize,
    pub escape_count: usize, // For Tactical Reset
}

impl TosState {
    pub fn new() -> Self {
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
            }],
            active_hub_index: 0,
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
            }],
            active_hub_index: 0,
        };

        let initial_viewport = Viewport {
            id: uuid::Uuid::new_v4(),
            sector_index: 0,
            hub_index: 0,
            current_level: HierarchyLevel::GlobalOverview,
            active_app_index: None,
        };

        Self {
            current_level: HierarchyLevel::GlobalOverview,
            sectors: vec![first_sector, second_sector],
            viewports: vec![initial_viewport],
            active_viewport_index: 0,
            escape_count: 0,
        }
    }

    pub fn tactical_reset(&mut self) {
        self.current_level = HierarchyLevel::GlobalOverview;
        for viewport in &mut self.viewports {
            viewport.current_level = HierarchyLevel::GlobalOverview;
        }
        self.escape_count = 0;
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
        if viewport.current_level == HierarchyLevel::CommandHub {
            let sector = &mut self.sectors[viewport.sector_index];
            let hub = &mut sector.hubs[viewport.hub_index];
            hub.prompt = cmd;
        }
    }

    pub fn render_current_view(&self) -> String {
        if self.viewports.len() > 1 {
            self.render_split_view()
        } else {
            let viewport = &self.viewports[0];
            self.render_viewport(viewport)
        }
    }

    fn render_viewport(&self, viewport: &Viewport) -> String {
        match viewport.current_level {
            HierarchyLevel::GlobalOverview => self.render_global_overview(),
            HierarchyLevel::CommandHub => self.render_command_hub(viewport),
            HierarchyLevel::ApplicationFocus => self.render_application_focus(viewport),
            HierarchyLevel::SplitView => self.render_split_view(),
            HierarchyLevel::DetailInspector => self.render_detail_inspector(viewport),
            HierarchyLevel::BufferInspector => self.render_buffer_inspector(viewport),
        }
    }

    fn render_global_overview(&self) -> String {
        let mut html = String::from(r#"<div class="global-grid">"#);
        for (i, sector) in self.sectors.iter().enumerate() {
            html.push_str(&format!(
                r#"<div class="sector-card" style="border-left-color: {color}" onclick="window.ipc.postMessage('select_sector:{index}')">
                    <div class="sector-meta">SECTOR {index}</div>
                    <div class="sector-name">{name}</div>
                    <div class="sector-stats">{hubs} HUBS // {apps} APPS</div>
                </div>"#,
                color = sector.color,
                index = i,
                name = sector.name,
                hubs = sector.hubs.len(),
                apps = sector.hubs.iter().map(|h| h.applications.len()).sum::<usize>()
            ));
        }
        html.push_str("</div>");
        html
    }

    fn render_command_hub(&self, viewport: &Viewport) -> String {
        let sector = &self.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        
        let mode_class = match hub.mode {
            CommandHubMode::Command => "mode-command",
            CommandHubMode::Directory => "mode-directory",
            CommandHubMode::Activity => "mode-activity",
        };

        let mut html = format!(r#"<div class="command-hub {mode_class}">"#);
        
        html.push_str(r#"<div class="hub-tabs">"#);
        let modes = [
            (CommandHubMode::Command, "COMMAND"),
            (CommandHubMode::Directory, "DIRECTORY"),
            (CommandHubMode::Activity, "ACTIVITY"),
        ];
        for (m, label) in modes {
            let active = if hub.mode == m { "active" } else { "" };
            html.push_str(&format!(
                r#"<div class="hub-tab {active}" onclick="window.ipc.postMessage('set_mode:{mode:?}')">{label}</div>"#,
                mode = m
            ));
        }
        html.push_str("</div>");

        html.push_str(r#"<div class="hub-content">"#);
        match hub.mode {
            CommandHubMode::Command => {
                let mut output_html = String::new();
                for line in &hub.terminal_output {
                    output_html.push_str(&format!(r#"<div class="log-line">{}</div>"#, line));
                }

                html.push_str(&format!(r#"<div class="command-view">
                    <div class="terminal-output" id="hub-term-{}">
                        {}
                    </div>
                </div>"#, hub.id, output_html));
            }
            CommandHubMode::Directory => {
                html.push_str(r#"<div class="directory-view">
                    <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:ls ..')">..</div>
                    <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:cd DOCUMENTS')">DOCUMENTS/</div>
                    <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:ls SYSTEM_CORE')">SYSTEM_CORE/</div>
                    <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:view CONFIG.TOS')">CONFIG.TOS</div>
                </div>"#);
            }
            CommandHubMode::Activity => {
                html.push_str(r#"<div class="activity-view">"#);
                for app in &hub.applications {
                    html.push_str(&format!(
                        r#"<div class="app-tile staging-item" onclick="window.ipc.postMessage('stage_command:focus {title}')">
                            <div class="app-title">{title}</div>
                            <div class="app-class">{class}</div>
                        </div>"#,
                        title = app.title,
                        class = app.app_class
                    ));
                }
                html.push_str("</div>");
            }
        }
        html.push_str("</div>");

        html.push_str(&format!(
            r#"<div class="unified-prompt">
                <div class="prompt-prefix">TOS@{} ></div>
                <input type="text" id="terminal-input" value="{}" onkeydown="handlePromptKey(event)" autofocus>
            </div>"#,
            sector.name.to_uppercase(), hub.prompt
        ));

        html.push_str("</div>");
        html
    }

    fn render_application_focus(&self, viewport: &Viewport) -> String {
        let sector = &self.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];

        format!(
            r#"<div class="application-container">
                <div class="tactical-bezel">
                    <div class="bezel-top">
                        <div class="bezel-back" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
                        <div class="bezel-title">{title} // {class}</div>
                        <div class="bezel-status" onclick="window.ipc.postMessage('split_viewport')">SPLIT</div>
                    </div>
                </div>
                <div class="application-surface" onclick="window.ipc.postMessage('zoom_in')">
                    <div class="app-mock-content">
                        APPLICATION DATA FEED: {title}
                    </div>
                </div>
            </div>"#,
            title = app.title, class = app.app_class
        )
    }

    fn render_detail_inspector(&self, viewport: &Viewport) -> String {
        let sector = &self.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];

        format!(
            r#"<div class="inspector-container detail-inspector">
                <div class="inspector-header">NODE INSPECTOR // LEVEL 4</div>
                <div class="inspector-content">
                    <div class="stat-row"><span>ID:</span> <span>{id}</span></div>
                    <div class="stat-row"><span>CLASS:</span> <span>{class}</span></div>
                    <div class="stat-row"><span>SECTOR:</span> <span>{sector}</span></div>
                    <div class="stat-row"><span>PERMISSIONS:</span> <span>0755</span></div>
                    <div class="stat-row"><span>UPTIME:</span> <span>00:14:32</span></div>
                </div>
                <div class="inspector-footer" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
            </div>"#,
            id = app.id, class = app.app_class, sector = sector.name
        )
    }

    fn render_buffer_inspector(&self, _viewport: &Viewport) -> String {
        r#"<div class="inspector-container buffer-inspector">
            <div class="inspector-header">BUFFER HEX DUMP // LEVEL 5</div>
            <div class="buffer-hex">
                0000: 4c 43 41 52 53 20 44 52 45 41 4d 20 43 4f 4d 50  LCARS DREAM COMP
                0010: 4c 45 54 45 20 56 45 52 53 49 4f 4e 20 31 2e 30  LETE VERSION 1.0
                0020: 0a 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f  ................
            </div>
            <div class="inspector-footer" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
        </div>"#.to_string()
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
        assert_eq!(state.sectors.len(), 2);
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
}
