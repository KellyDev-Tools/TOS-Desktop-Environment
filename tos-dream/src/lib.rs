use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchyLevel {
    GlobalOverview,
    CommandHub,
    ApplicationFocus,
    SplitView,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHub {
    pub id: uuid::Uuid,
    pub mode: CommandHubMode,
    pub prompt: String,
    pub applications: Vec<Application>,
    pub active_app_index: Option<usize>,
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
        }
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
                html.push_str(&format!(r#"<div class="command-view">
                    <div class="terminal-output">
                        <div class="log-line">SYSTEM INITIALIZED // MISSION LOG START</div>
                        <div class="log-line">SECTOR: {name}</div>
                        <div class="log-line">STATION: HUB {hub_idx}</div>
                    </div>
                </div>"#, name = sector.name.to_uppercase(), hub_idx = viewport.hub_index));
            }
            CommandHubMode::Directory => {
                html.push_str(r#"<div class="directory-view">
                    <div class="file-item">..</div>
                    <div class="file-item">DOCUMENTS/</div>
                    <div class="file-item">SYSTEM_CORE/</div>
                    <div class="file-item">CONFIG.TOS</div>
                </div>"#);
            }
            CommandHubMode::Activity => {
                html.push_str(r#"<div class="activity-view">"#);
                for app in &hub.applications {
                    html.push_str(&format!(
                        r#"<div class="app-tile" onclick="window.ipc.postMessage('focus_app:{app_id}')">
                            <div class="app-title">{title}</div>
                            <div class="app-class">{class}</div>
                        </div>"#,
                        app_id = app.id,
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
                <div class="application-surface">
                    <div class="app-mock-content">
                        APPLICATION DATA FEED: {title}
                    </div>
                </div>
            </div>"#,
            title = app.title, class = app.app_class
        )
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

        // Zoom back out to Hub
        state.zoom_out();
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);

        // Zoom back out to Global
        state.zoom_out();
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    }

    #[test]
    fn test_mode_switching() {
        let mut state = TosState::new();
        state.zoom_in(); // Go to Hub
        
        state.toggle_mode(CommandHubMode::Directory);
        let sector = &state.sectors[state.viewports[0].sector_index];
        let hub = &sector.hubs[state.viewports[0].hub_index];
        assert_eq!(hub.mode, CommandHubMode::Directory);
    }

    #[test]
    fn test_split_logic() {
        let mut state = TosState::new();
        state.zoom_in(); // Hub
        state.zoom_in(); // App
        
        // Manual split simulation (main.rs logic)
        let sector_idx = state.viewports[0].sector_index;
        let hub_idx = state.viewports[0].hub_index;
        state.viewports.push(Viewport {
            id: uuid::Uuid::new_v4(),
            sector_index: sector_idx,
            hub_index: hub_idx,
            current_level: HierarchyLevel::CommandHub,
            active_app_index: None,
        });
        state.current_level = HierarchyLevel::SplitView;

        assert_eq!(state.viewports.len(), 2);
        assert_eq!(state.current_level, HierarchyLevel::SplitView);
        
        // Verify render doesn't crash
        let html = state.render_current_view();
        assert!(html.contains("split-viewport-grid"));
    }
}
