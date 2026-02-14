pub mod system;
use system::input::SemanticEvent;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: uuid::Uuid,
    pub name: String,
    pub color: String,
    pub hubs: Vec<CommandHub>,
    pub active_hub_index: usize,
    pub host: String,
    pub is_remote: bool,
    pub participants: Vec<Participant>,
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
                confirmation_required: None,
            }],
            active_hub_index: 0,
            host: "LOCAL".to_string(),
            is_remote: false,
            participants: vec![Participant { name: "Host".to_string(), color: "#ffcc00".to_string(), role: "Co-owner".to_string() }],
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
            is_remote: true,
            participants: vec![
                Participant { name: "Commander".to_string(), color: "#ffcc00".to_string(), role: "Co-owner".to_string() },
                Participant { name: "Ensign Kim".to_string(), color: "#99ccff".to_string(), role: "Operator".to_string() },
                Participant { name: "Seven".to_string(), color: "#cc99ff".to_string(), role: "Viewer".to_string() },
            ],
        };

        let initial_viewport = Viewport {
            id: uuid::Uuid::new_v4(),
            sector_index: 0,
            hub_index: 0,
            current_level: HierarchyLevel::GlobalOverview,
            active_app_index: None,
            bezel_expanded: false,
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

    pub fn toggle_bezel(&mut self) {
        self.viewports[self.active_viewport_index].bezel_expanded = !self.viewports[self.active_viewport_index].bezel_expanded;
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

    pub fn handle_semantic_event(&mut self, event: SemanticEvent) {
        match event {
            SemanticEvent::ZoomIn => self.zoom_in(),
            SemanticEvent::ZoomOut => self.zoom_out(),
            SemanticEvent::TacticalReset => self.tactical_reset(),
            SemanticEvent::ToggleBezel => self.toggle_bezel(),
            SemanticEvent::ModeCommand => self.toggle_mode(CommandHubMode::Command),
            SemanticEvent::ModeDirectory => self.toggle_mode(CommandHubMode::Directory),
            SemanticEvent::ModeActivity => self.toggle_mode(CommandHubMode::Activity),
            SemanticEvent::CycleMode => {
                let viewport = &self.viewports[self.active_viewport_index];
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
            let remote_class = if sector.is_remote { "is-remote" } else { "is-local" };
            html.push_str(&format!(
                r#"<div class="sector-card {remote_class}" style="border-left-color: {color}" onclick="window.ipc.postMessage('select_sector:{index}')">
                    <div class="sector-meta">SECTOR {index} // {host}</div>
                    <div class="sector-name">{name}</div>
                    <div class="sector-stats">{hubs} HUBS // {apps} APPS</div>
                </div>"#,
                color = sector.color,
                index = i,
                host = sector.host,
                name = sector.name,
                hubs = sector.hubs.len(),
                apps = sector.hubs.iter().map(|h| h.applications.len()).sum::<usize>()
            ));
        }
        
        html.push_str(r#"<div class="sector-card add-remote-card" onclick="window.ipc.postMessage('add_remote_sector')">
            <div class="sector-meta">SYS // COMMAND</div>
            <div class="sector-name">+ ADD REMOTE</div>
            <div class="sector-stats">ESTABLISH NEW SECTOR LINK</div>
        </div>"#);

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

        let mut participants_html = String::new();
        for p in &sector.participants {
            participants_html.push_str(&format!(
                r#"<div class="participant-avatar" style="background-color: {color}" title="{name} ({role})"></div>"#,
                color = p.color, name = p.name, role = p.role
            ));
        }

        html.push_str(&format!(
            r#"<div class="hub-header">
                <div class="hub-info">
                    <span class="hub-sector-name">{name}</span>
                    <span class="hub-host">LINK: {host}</span>
                </div>
                <div class="hub-participants">
                    {participants_html}
                    <div class="invite-btn" onclick="window.ipc.postMessage('collaboration_invite')">+</div>
                </div>
            </div>"#,
            name = sector.name.to_uppercase(),
            host = sector.host,
            participants_html = participants_html
        ));
        
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
        if let Some(dangerous_cmd) = &hub.confirmation_required {
            html.push_str(&format!(
                r#"<div class="dangerous-overlay">
                    <div class="alert-header">TACTICAL ALERT // DANGEROUS COMMAND DETECTED</div>
                    <div class="alert-subline">EXECUTION BLOCKED PENDING TACTILE CONFIRMATION</div>
                    <div class="dangerous-command">SPEC: {cmd}</div>
                    <div class="confirmation-zone">
                        <div class="slider-track">
                            <input type="range" class="confirm-slider" min="0" max="100" value="0" 
                                oninput="if(this.value == 100) {{ window.ipc.postMessage('prompt_submit:{cmd}'); }}"
                                onchange="if(this.value < 100) {{ this.value = 0; }}">
                            <div class="slider-label">SLIDE TO CONFIRM EXECUTION</div>
                        </div>
                    </div>
                    <div class="bezel-btn danger" onclick="window.ipc.postMessage('stage_command:')">ABORT ACTION</div>
                </div>"#,
                cmd = dangerous_cmd
            ));
        }
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
                    <div class="path-bar">/HOME/USER/SECTOR_PRIMARY</div>
                    <div class="file-grid">
                        <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:ls ..')">..</div>
                        <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:cd DOCUMENTS')">DOCUMENTS/</div>
                        <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:ls SYSTEM_CORE')">SYSTEM_CORE/</div>
                        <div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:view CONFIG.TOS')">CONFIG.TOS</div>
                    </div>
                </div>"#);
            }
            CommandHubMode::Activity => {
                let mut apps_html = String::new();
                for app in &hub.applications {
                    apps_html.push_str(&format!(
                        r#"<div class="app-tile staging-item" onclick="window.ipc.postMessage('stage_command:focus {title}')">
                            <div class="app-tile-icon"></div>
                            <div class="app-tile-info">
                                <div class="app-title">{title}</div>
                                <div class="app-class">{class}</div>
                            </div>
                            <div class="app-tile-stats">
                                <div class="stat">CPU: 2.1%</div>
                                <div class="stat">MEM: 82MB</div>
                            </div>
                        </div>"#,
                        title = app.title.to_uppercase(),
                        class = app.app_class.to_uppercase()
                    ));
                }
                html.push_str(&format!(
                    r#"<div class="activity-view">
                        <div class="activity-grid">
                            {apps_html}
                            <div class="app-tile add-tile" onclick="window.ipc.postMessage('stage_command:spawn ')">
                                <span>+ NEW PROCESS</span>
                            </div>
                        </div>
                    </div>"#,
                    apps_html = apps_html
                ));
            }
        }
        html.push_str("</div>");

        html.push_str(&format!(
            r#"<div class="unified-prompt">
                <div class="voice-trigger" onclick="window.ipc.postMessage('semantic_event:VoiceCommandStart')">
                    <span class="mic-icon"></span>
                </div>
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
        let bezel_class = if viewport.bezel_expanded { "expanded" } else { "collapsed" };

        let mut participants_html = String::new();
        for p in &sector.participants {
            participants_html.push_str(&format!(
                r#"<div class="participant-avatar mini" style="background-color: {color}" title="{name}"></div>"#,
                color = p.color, name = p.name
            ));
        }

        format!(
            r#"<div class="application-container">
                <div class="tactical-bezel {bezel_class}">
                    <div class="bezel-top">
                        <div class="bezel-back" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
                        <div class="bezel-title">{title} // {class}</div>
                        <div class="bezel-participants">
                            {participants_html}
                        </div>
                        <div class="bezel-handle" onclick="window.ipc.postMessage('toggle_bezel')">
                            <span class="chevron"></span>
                        </div>
                    </div>
                    <div class="bezel-expanded-content">
                        <div class="bezel-group">
                            <div class="bezel-btn" onclick="window.ipc.postMessage('zoom_out')">ZOOM OUT</div>
                            <div class="bezel-btn" onclick="window.ipc.postMessage('split_viewport')">SPLIT VIEW</div>
                            <div class="bezel-btn">TELEPORT</div>
                            <div class="bezel-btn danger">CLOSE</div>
                        </div>
                        <div class="bezel-group sliders">
                            <div class="action-slider">
                                <span>PRIORITY</span>
                                <input type="range" min="1" max="10" value="5">
                            </div>
                            <div class="action-slider">
                                <span>POWER</span>
                                <input type="range" min="1" max="100" value="80">
                            </div>
                        </div>
                    </div>
                </div>
                <div class="application-surface" onclick="window.ipc.postMessage('zoom_in')">
                    <div class="app-mock-content">
                        APPLICATION DATA FEED: {title}
                    </div>
                </div>
            </div>"#,
            bezel_class = bezel_class,
            title = app.title.to_uppercase(),
            class = app.app_class.to_uppercase()
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
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    }
}
