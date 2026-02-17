//! Tactical Mini-Map Module
//! 
//! Provides an ephemeral overlay for spatial awareness without blocking interaction.
//! Shows current sector, other sectors (dimmed), viewports, and current depth.
//! 
//! ## Activation Methods (Configurable)
//! - Hover (dwell time)
//! - Keyboard shortcut (Ctrl+M)
//! - Modifier + click (Alt+click)
//! - Double-tap (touch)
//! - Game controller button
//! - Voice ("activate mini-map")
//! 
//! ## Content by Depth
//! - **Level 1**: All sectors as miniature tiles
//! - **Level 2**: Current sector with mode indicator; other sectors dimmed
//! - **Level 3**: Current sector with focused app highlighted; other viewports shown

use crate::{HierarchyLevel, TosState, CommandHubMode};
use serde::{Deserialize, Serialize};

/// Position of the mini-map on screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniMapPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

/// Activation method for the mini-map
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationMethod {
    /// Hover with dwell time in milliseconds
    Hover(u64),
    /// Keyboard shortcut (e.g., "Ctrl+M")
    KeyboardShortcut(String),
    /// Modifier key + click
    ModifierClick(String), // e.g., "Alt"
    /// Double-tap for touch
    DoubleTap,
    /// Game controller button
    GamepadButton(String),
    /// Voice command
    Voice(String),
}

/// Configuration for the mini-map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniMapConfig {
    /// Position on screen
    pub position: MiniMapPosition,
    /// Size as percentage of screen (width, height)
    pub size: (f32, f32),
    /// Opacity when passive (0.0 - 1.0)
    pub passive_opacity: f32,
    /// Opacity when active (0.0 - 1.0)
    pub active_opacity: f32,
    /// Activation methods
    pub activation_methods: Vec<ActivationMethod>,
    /// Whether to show other sectors
    pub show_other_sectors: bool,
    /// Whether to show viewport dividers
    pub show_viewports: bool,
    /// Dwell time for hover activation (ms)
    pub hover_dwell_ms: u64,
}

impl Default for MiniMapConfig {
    fn default() -> Self {
        Self {
            position: MiniMapPosition::BottomRight,
            size: (0.2, 0.25), // 20% width, 25% height
            passive_opacity: 0.3,
            active_opacity: 0.9,
            activation_methods: vec![
                ActivationMethod::Hover(1000),
                ActivationMethod::KeyboardShortcut("Ctrl+M".to_string()),
                ActivationMethod::Voice("activate mini-map".to_string()),
            ],
            show_other_sectors: true,
            show_viewports: true,
            hover_dwell_ms: 1000,
        }
    }
}

/// State of the mini-map
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniMapState {
    /// Passive state - input passes through
    Passive,
    /// Active state - captures input
    Active,
    /// Currently being hovered (for dwell activation)
    Hovering(std::time::Instant),
}

/// The Tactical Mini-Map
#[derive(Debug)]
pub struct MiniMap {
    /// Configuration
    pub config: MiniMapConfig,
    /// Current state
    pub state: MiniMapState,
    /// Last hover position (for dwell tracking)
    pub last_hover_pos: Option<(f32, f32)>,
    /// Selected sector index (when active)
    pub selected_sector: Option<usize>,
    /// Selected viewport index (when active)
    pub selected_viewport: Option<usize>,
}

impl Default for MiniMap {
    fn default() -> Self {
        Self::new()
    }
}

impl MiniMap {
    /// Create a new mini-map with default configuration
    pub fn new() -> Self {
        Self::with_config(MiniMapConfig::default())
    }

    /// Create a new mini-map with custom configuration
    pub fn with_config(config: MiniMapConfig) -> Self {
        Self {
            config,
            state: MiniMapState::Passive,
            last_hover_pos: None,
            selected_sector: None,
            selected_viewport: None,
        }
    }

    /// Check if the mini-map is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.state, MiniMapState::Active)
    }

    /// Activate the mini-map
    pub fn activate(&mut self) {
        self.state = MiniMapState::Active;
    }

    /// Deactivate the mini-map (return to passive)
    pub fn deactivate(&mut self) {
        self.state = MiniMapState::Passive;
        self.selected_sector = None;
        self.selected_viewport = None;
    }

    /// Toggle active/passive state
    pub fn toggle(&mut self) {
        if self.is_active() {
            self.deactivate();
        } else {
            self.activate();
        }
    }

    /// Handle hover event for dwell activation
    pub fn handle_hover(&mut self, x: f32, y: f32) {
        match self.state {
            MiniMapState::Passive => {
                // Check if hover position changed significantly
                if let Some((last_x, last_y)) = self.last_hover_pos {
                    let dx = x - last_x;
                    let dy = y - last_y;
                    if dx.abs() > 5.0 || dy.abs() > 5.0 {
                        // Movement detected, reset dwell timer
                        self.state = MiniMapState::Hovering(std::time::Instant::now());
                    }
                } else {
                    self.state = MiniMapState::Hovering(std::time::Instant::now());
                }
                
                // Check if dwell time exceeded
                if let MiniMapState::Hovering(start) = self.state {
                    if start.elapsed().as_millis() as u64 >= self.config.hover_dwell_ms {
                        self.activate();
                    }
                }
            }
            MiniMapState::Active => {
                // In active mode, track position for selection
                self.update_selection(x, y);
            }
            _ => {}
        }
        
        self.last_hover_pos = Some((x, y));
    }

    /// Handle hover exit
    pub fn handle_hover_exit(&mut self) {
        self.last_hover_pos = None;
        if matches!(self.state, MiniMapState::Hovering(_)) {
            self.state = MiniMapState::Passive;
        }
    }

    /// Update selection based on position (when active)
    fn update_selection(&mut self, _x: f32, _y: f32) {
        // This would calculate which sector/viewport is under the cursor
        // For now, just store the position for rendering highlight
        // Actual selection happens on click
    }

    /// Handle click when active - returns selected navigation target
    pub fn handle_click(&self, x: f32, y: f32, state: &TosState) -> Option<NavigationTarget> {
        if !self.is_active() {
            return None;
        }

        // Calculate which element was clicked
        let (sector_idx, viewport_idx) = self.calculate_click_target(x, y, state)?;
        
        Some(NavigationTarget {
            sector_index: sector_idx,
            viewport_index: viewport_idx,
        })
    }

    /// Calculate what was clicked based on position
    fn calculate_click_target(&self, x: f32, y: f32, state: &TosState) -> Option<(usize, Option<usize>)> {
        // Simplified layout calculation
        // In a real implementation, this would use actual layout geometry
        
        let num_sectors = state.sectors.len();
        if num_sectors == 0 {
            return None;
        }

        // Determine layout based on current level
        match state.current_level {
            HierarchyLevel::GlobalOverview => {
                // Grid of all sectors
                let cols = (num_sectors as f32).sqrt().ceil() as usize;
                let cell_width = 1.0 / cols as f32;
                let cell_height = 1.0 / ((num_sectors + cols - 1) / cols) as f32;
                
                let col = (x / cell_width) as usize;
                let row = (y / cell_height) as usize;
                let idx = row * cols + col;
                
                if idx < num_sectors {
                    Some((idx, None))
                } else {
                    None
                }
            }
            _ => {
                // Show current sector with viewports
                let viewport = &state.viewports[state.active_viewport_index];
                let sector_idx = viewport.sector_index;
                
                if state.viewports.len() > 1 && self.config.show_viewports {
                    // Show viewport grid
                    let vp_cols = (state.viewports.len() as f32).sqrt().ceil() as usize;
                    let cell_width = 1.0 / vp_cols as f32;
                    let cell_height = 1.0 / ((state.viewports.len() + vp_cols - 1) / vp_cols) as f32;
                    
                    let col = (x / cell_width) as usize;
                    let row = (y / cell_height) as usize;
                    let vp_idx = row * vp_cols + col;
                    
                    if vp_idx < state.viewports.len() {
                        Some((sector_idx, Some(vp_idx)))
                    } else {
                        Some((sector_idx, None))
                    }
                } else {
                    Some((sector_idx, None))
                }
            }
        }
    }

    /// Render the mini-map as HTML
    pub fn render(&self, state: &TosState) -> String {
        let opacity = if self.is_active() {
            self.config.active_opacity
        } else {
            self.config.passive_opacity
        };

        let position_class = match self.config.position {
            MiniMapPosition::TopLeft => "minimap-topleft",
            MiniMapPosition::TopRight => "minimap-topright",
            MiniMapPosition::BottomLeft => "minimap-bottomleft",
            MiniMapPosition::BottomRight => "minimap-bottomright",
            MiniMapPosition::Center => "minimap-center",
        };

        let state_class = if self.is_active() {
            "minimap-active"
        } else {
            "minimap-passive"
        };

        let content = match state.current_level {
            HierarchyLevel::GlobalOverview => self.render_global_overview(state),
            HierarchyLevel::CommandHub => self.render_command_hub(state),
            HierarchyLevel::ApplicationFocus | HierarchyLevel::DetailInspector | HierarchyLevel::BufferInspector => {
                self.render_application_focus(state)
            }
            HierarchyLevel::SplitView => self.render_split_view(state),
        };

        format!(
            r#"<div class="tactical-minimap {} {}" style="opacity: {};">
                <div class="minimap-header">TACTICAL MINI-MAP</div>
                <div class="minimap-content">
                    {}
                </div>
                <div class="minimap-footer">{}</div>
            </div>"#,
            position_class,
            state_class,
            opacity,
            content,
            self.render_legend(state)
        )
    }

    /// Render global overview (Level 1) - all sectors
    fn render_global_overview(&self, state: &TosState) -> String {
        let mut html = String::from(r#"<div class="minimap-sectors-grid">"#);
        
        for (idx, sector) in state.sectors.iter().enumerate() {
            let is_active = idx == state.viewports[state.active_viewport_index].sector_index;
            let dim_class = if is_active { "minimap-sector-active" } else { "minimap-sector-dimmed" };
            
            html.push_str(&format!(
                r#"<div class="minimap-sector {} {}" data-sector="{}" style="border-color: {};">
                    <div class="sector-name">{}</div>
                    <div class="sector-host">{}</div>
                    <div class="sector-hubs">{} hubs</div>
                </div>"#,
                dim_class,
                if self.selected_sector == Some(idx) { "minimap-selected" } else { "" },
                idx,
                sector.color,
                sector.name,
                sector.host,
                sector.hubs.len()
            ));
        }
        
        html.push_str("</div>");
        html
    }

    /// Render command hub view (Level 2) - current sector with mode
    fn render_command_hub(&self, state: &TosState) -> String {
        let viewport = &state.viewports[state.active_viewport_index];
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        
        let (mode_icon, mode_name) = match hub.mode {
            CommandHubMode::Command => ("⌘", "Command"),
            CommandHubMode::Directory => ("📁", "Directory"),
            CommandHubMode::Activity => ("⚡", "Activity"),
        };

        let mut html = format!(
            r#"<div class="minimap-current-sector">
                <div class="sector-header" style="border-color: {};">
                    <span class="sector-indicator">◉</span>
                    <span class="sector-name">{}</span>
                </div>
                <div class="hub-info">
                    <div class="hub-mode">{} {} Mode</div>
                    <div class="hub-apps">{} applications</div>
                </div>
            </div>"#,
            sector.color,
            sector.name,
            mode_icon,
            mode_name,
            hub.applications.len()
        );

        // Show other sectors dimmed
        if self.config.show_other_sectors {
            html.push_str(r#"<div class="minimap-other-sectors">"#);
            for (idx, other_sector) in state.sectors.iter().enumerate() {
                if idx != viewport.sector_index {
                    html.push_str(&format!(
                        r#"<div class="minimap-sector-dimmed" data-sector="{}">
                            <span style="color: {};">◉</span> {}
                        </div>"#,
                        idx,
                        other_sector.color,
                        other_sector.name
                    ));
                }
            }
            html.push_str("</div>");
        }

        html
    }

    /// Render application focus view (Level 3) - current app with viewports
    fn render_application_focus(&self, state: &TosState) -> String {
        let viewport = &state.viewports[state.active_viewport_index];
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        
        let active_app = viewport.active_app_index
            .and_then(|idx| hub.applications.get(idx))
            .map(|app| app.title.clone())
            .unwrap_or_else(|| "No Application".to_string());

        let mut html = format!(
            r#"<div class="minimap-app-focus">
                <div class="app-path">
                    <span style="color: {};">{}</span> → {} → <strong>{}</strong>
                </div>
                <div class="active-app">
                    <div class="app-title">{}</div>
                    <div class="app-class">{}</div>
                </div>
            </div>"#,
            sector.color,
            sector.name,
            match hub.mode {
                CommandHubMode::Command => "Command",
                CommandHubMode::Directory => "Directory",
                CommandHubMode::Activity => "Activity",
            },
            active_app,
            active_app,
            viewport.active_app_index
                .and_then(|idx| hub.applications.get(idx))
                .map(|app| app.app_class.clone())
                .unwrap_or_default()
        );

        // Show viewports if in split mode
        if state.viewports.len() > 1 && self.config.show_viewports {
            html.push_str(r#"<div class="minimap-viewports">"#);
            for (idx, vp) in state.viewports.iter().enumerate() {
                let is_active = idx == state.active_viewport_index;
                let vp_class = if is_active { "viewport-active" } else { "viewport-inactive" };
                
                html.push_str(&format!(
                    r#"<div class="minimap-viewport {} {}" data-viewport="{}">
                        <div class="viewport-id">VP{}</div>
                        <div class="viewport-sector">{}</div>
                    </div>"#,
                    vp_class,
                    if self.selected_viewport == Some(idx) { "minimap-selected" } else { "" },
                    idx,
                    idx + 1,
                    state.sectors[vp.sector_index].name
                ));
            }
            html.push_str("</div>");
        }

        html
    }

    /// Render split view
    fn render_split_view(&self, state: &TosState) -> String {
        let mut html = String::from(r#"<div class="minimap-split-view">
            <div class="split-indicator">SPLIT VIEW MODE</div>
            <div class="viewport-grid">"#);
        
        for (idx, vp) in state.viewports.iter().enumerate() {
            let is_active = idx == state.active_viewport_index;
            let sector = &state.sectors[vp.sector_index];
            
            html.push_str(&format!(
                r#"<div class="split-viewport {}">
                    <div class="vp-header" style="background: {};">VP{} {}</div>
                    <div class="vp-level">{:?}</div>
                </div>"#,
                if is_active { "active" } else { "" },
                sector.color,
                idx + 1,
                if is_active { "◉" } else { "" },
                vp.current_level
            ));
        }
        
        html.push_str("</div></div>");
        html
    }

    /// Render legend/footer
    fn render_legend(&self, state: &TosState) -> String {
        let level_desc = match state.current_level {
            HierarchyLevel::GlobalOverview => "Level 1: Global",
            HierarchyLevel::CommandHub => "Level 2: Hub",
            HierarchyLevel::ApplicationFocus => "Level 3: App",
            HierarchyLevel::DetailInspector => "Level 3+: Detail",
            HierarchyLevel::BufferInspector => "Level 3+: Buffer",
            HierarchyLevel::SplitView => "Split View",
        };

        if self.is_active() {
            format!("{} | Click to navigate | Press Ctrl+M to close", level_desc)
        } else {
            format!("{} | Hover or Ctrl+M to activate", level_desc)
        }
    }
}

/// Navigation target from mini-map click
#[derive(Debug, Clone)]
pub struct NavigationTarget {
    pub sector_index: usize,
    pub viewport_index: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_default_config() {
        let config = MiniMapConfig::default();
        assert_eq!(config.position, MiniMapPosition::BottomRight);
        assert_eq!(config.size, (0.2, 0.25));
        assert!(config.show_other_sectors);
        assert!(config.show_viewports);
    }

    #[test]
    fn test_minimap_state_transitions() {
        let mut minimap = MiniMap::new();
        
        assert!(!minimap.is_active());
        
        minimap.activate();
        assert!(minimap.is_active());
        
        minimap.deactivate();
        assert!(!minimap.is_active());
        
        minimap.toggle();
        assert!(minimap.is_active());
        
        minimap.toggle();
        assert!(!minimap.is_active());
    }

    #[test]
    fn test_activation_methods() {
        let config = MiniMapConfig::default();
        assert!(!config.activation_methods.is_empty());
        
        let has_hover = config.activation_methods.iter().any(|m| matches!(m, ActivationMethod::Hover(_)));
        let has_keyboard = config.activation_methods.iter().any(|m| matches!(m, ActivationMethod::KeyboardShortcut(_)));
        
        assert!(has_hover);
        assert!(has_keyboard);
    }

    #[test]
    fn test_position_classes() {
        let mut minimap = MiniMap::new();
        
        minimap.config.position = MiniMapPosition::TopLeft;
        let html = minimap.render(&crate::TosState::new());
        assert!(html.contains("minimap-topleft"));
        
        minimap.config.position = MiniMapPosition::BottomRight;
        let html = minimap.render(&crate::TosState::new());
        assert!(html.contains("minimap-bottomright"));
    }

    #[test]
    fn test_render_levels() {
        let mut state = crate::TosState::new();
        let minimap = MiniMap::new();
        
        // Level 1
        let html = minimap.render(&state);
        assert!(html.contains("TACTICAL MINI-MAP"));
        assert!(html.contains("minimap-sectors-grid"));
        
        // Level 2
        state.zoom_in();
        let html = minimap.render(&state);
        assert!(html.contains("minimap-current-sector"));
        
        // Level 3
        state.zoom_in();
        let html = minimap.render(&state);
        assert!(html.contains("minimap-app-focus"));
    }
}
