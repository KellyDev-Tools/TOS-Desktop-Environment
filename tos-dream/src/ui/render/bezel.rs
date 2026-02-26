use crate::{TosState, Viewport, HierarchyLevel};
use crate::system::collaboration::Participant;
use crate::system::collaboration::PermissionAction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BezelState {
    Collapsed,
    Expanded,
}

/// Render collaboration avatars with priority for the local participant.
fn render_avatars(participants: &[Participant], local_id: uuid::Uuid) -> String {
    let mut html = String::new();
    for p in participants {
        let is_local = p.id == local_id;
        let local_class = if is_local { "local" } else { "" };
        let initials = p.name.chars().next().unwrap_or('?').to_uppercase().to_string();
        
        let content = if let Some(url) = &p.avatar_url {
            format!(r#"<img src="{}" alt="{}" />"#, url, p.name)
        } else {
            format!(r#"<span>{}</span>"#, initials)
        };

        html.push_str(&format!(
            r#"<div class="collab-avatar {local_class}" style="--avatar-color:{};" title="{} ({})">
                {}
            </div>"#,
            p.color, p.name, p.role.as_str(), content
        ));
    }
    html
}

/// Render the tactical bezel based on hierarchy level and expansion state.
pub fn render_bezel(state: &TosState, viewport: &Viewport, level: HierarchyLevel, bezel_state: BezelState) -> String {
    let sector = &state.sectors[viewport.sector_index];
    let accent_color = &sector.color;
    let local_id = state.local_participant_id;
    
    let can_switch_mode = state.can_perform(PermissionAction::ModeSwitch);
    let can_manage_sector = state.can_perform(PermissionAction::SectorReset); // Using Reset as proxy for management
    let disabled_attr = |allowed: bool| if allowed { "" } else { "disabled" };
    
    let avatars_html = render_avatars(&sector.participants, local_id);
    let (title, icon) = if level == HierarchyLevel::ApplicationFocus {
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];
        (format!("{} // {}", app.title.to_uppercase(), app.app_class.to_uppercase()), app.icon.as_ref().unwrap_or(&sector.icon).clone())
    } else {
        (sector.name.to_uppercase(), sector.icon.clone())
    };
    let priority = sector.priority_score(state);
    let priority_class = match priority {
        p if p >= 8.0 => "priority-high",
        p if p >= 5.0 => "priority-medium",
        _ => "priority-low",
    };
    
    match bezel_state {
        BezelState::Collapsed => {
            match level {
                HierarchyLevel::ApplicationFocus => {
                    // BZ-02: L3: Application Bezel (Collapsed)
                    format!(
                        r#"<div class="tactical-bezel collapsed l3-bezel {priority_class}" style="--bezel-accent:{color};">
                            <div class="bezel-chips top">
                                <div class="chip"></div><div class="chip"></div>
                            </div>
                            <div class="bezel-top">
                                <div class="bezel-left">
                                    <span class="bezel-icon">{icon}</span>
                                    <span class="bezel-title">{title}</span>
                                </div>
                                <div class="bezel-right">
                                    <button class="bezel-btn mini zoom-out" onclick="window.ipc.postMessage('zoom_out')" title="Zoom Out (ESC)">▲</button>
                                    <div class="bezel-handle" onclick="window.ipc.postMessage('toggle_bezel')">
                                        <div class="chevron"></div>
                                    </div>
                                </div>
                            </div>
                            <div class="bezel-chips bottom">
                                <div class="chip"></div>
                            </div>
                        </div>"#,
                        priority_class = priority_class,
                        color = accent_color,
                        icon = icon,
                        title = title
                    )
                }
                HierarchyLevel::CommandHub => {
                    // BZ-03: L2: Command Hub Bezel (Output Mode)
                    let hub = &sector.hubs[viewport.hub_index];
                    
                    let mut suggestions_html = String::new();
                    for sug in &hub.suggestions {
                        suggestions_html.push_str(&format!(
                            r#"<div class="suggestion-item" onclick="window.ipc.postMessage('stage_command:{}')">
                                <span class="suggestion-cmd">{}</span>
                                <span class="suggestion-desc">{}</span>
                            </div>"#,
                            sug.command.replace('\'', "\\'"),
                            sug.text,
                            sug.description
                        ));
                    }

                    let drawer_active_class = if hub.suggestions.is_empty() { "" } else { "active" };
                    let cmd_active = if hub.mode == crate::CommandHubMode::Command { "active" } else { "" };
                    let dir_active = if hub.mode == crate::CommandHubMode::Directory { "active" } else { "" };
                    let act_active = if hub.mode == crate::CommandHubMode::Activity { "active" } else { "" };

                    format!(
                        r#"<div class="tactical-bezel collapsed hub-bezel {priority_class}" style="--bezel-accent:{color};">
                            <div class="bezel-chips top">
                                <div class="chip"></div><div class="chip"></div><div class="chip"></div>
                            </div>
                            <div class="bezel-top">
                                <div class="bezel-left">
                                    <button class="bezel-btn mini toggle-left-region" onclick="window.ipc.postMessage('toggle_left_region')" title="Toggle Region">◧</button>
                                    <div class="three-way-toggle">
                                        <div class="toggle-segment {cmd_active} {disabled_switch}" onclick="{cmd_onclick}">CMD</div>
                                        <div class="toggle-segment {dir_active} {disabled_switch}" onclick="{dir_onclick}">DIR</div>
                                        <div class="toggle-segment {act_active} {disabled_switch}" onclick="{act_onclick}">ACT</div>
                                    </div>
                                </div>
                                <div class="bezel-center">
                                    <span class="bezel-title">{title}</span>
                                </div>
                                <div class="bezel-right">
                                    <div class="collaboration-avatars">
                                        {avatars}
                                    </div>
                                    <button class="bezel-btn mini toggle-output-mode" onclick="window.ipc.postMessage('toggle_output_mode')" title="Perspective Mode">⬚</button>
                                    <div class="bezel-handle" onclick="window.ipc.postMessage('toggle_bezel')">
                                        <div class="chevron"></div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="autocomplete-drawer {drawer_active_class}">
                            <div class="drawer-header">TACTICAL AUTOCOMPLETE // SUGGESTIONS</div>
                            <div class="drawer-content">
                                {suggestions}
                            </div>
                        </div>"#,
                        priority_class = priority_class,
                        color = accent_color,
                        title = title,
                        cmd_active = cmd_active,
                        dir_active = dir_active,
                        act_active = act_active,
                        disabled_switch = if can_switch_mode { "" } else { "disabled" },
                        cmd_onclick = if can_switch_mode { "window.ipc.postMessage('set_mode:Command')" } else { "" },
                        dir_onclick = if can_switch_mode { "window.ipc.postMessage('set_mode:Directory')" } else { "" },
                        act_onclick = if can_switch_mode { "window.ipc.postMessage('set_mode:Activity')" } else { "" },
                        drawer_active_class = drawer_active_class,
                        suggestions = suggestions_html,
                        avatars = avatars_html
                    )
                }
                HierarchyLevel::GlobalOverview => {
                    // BZ-04: L1: Global Overview Bezel

                    format!(
                        r#"<div class="tactical-bezel collapsed global-bezel {priority_class}" style="--bezel-accent:{color};">
                            <div class="bezel-top">
                                <div class="bezel-left">
                                    <button class="bezel-btn mini settings-gear" onclick="window.ipc.postMessage('open_settings')" title="Settings" {disabled_manage}>⚙</button>
                                    <span class="bezel-title">GLOBAL OVERVIEW</span>
                                </div>
                                <div class="bezel-right">
                                    <div class="collaboration-avatars">
                                        {avatars}
                                    </div>
                                    <button class="bezel-btn mini add-sector" onclick="window.ipc.postMessage('add_sector')" title="Add Sector" {disabled_manage}>+</button>
                                    <div class="bezel-handle" onclick="window.ipc.postMessage('toggle_bezel')">
                                        <div class="chevron"></div>
                                    </div>
                                </div>
                            </div>
                        </div>"#,
                        priority_class = priority_class,
                        color = accent_color,
                        avatars = avatars_html,
                        disabled_manage = disabled_attr(can_manage_sector)
                    )
                }
                _ => {
                    format!(
                        r#"<div class="tactical-bezel collapsed" style="--bezel-accent:{color};">
                            <div class="bezel-top">
                                <span class="bezel-title">{title}</span>
                                <div class="bezel-handle" onclick="window.ipc.postMessage('toggle_bezel')">
                                    <div class="chevron"></div>
                                </div>
                            </div>
                        </div>"#,
                        color = accent_color,
                        title = title
                    )
                }
            }
        }
        BezelState::Expanded => {
            // BZ-08 Tactile Slider Implementation
            let hub = &sector.hubs[viewport.hub_index];
            
            let mut slider_html = String::new();
            if let Some(dangerous_cmd) = &hub.confirmation_required {
                slider_html = format!(
                    r#"<div class="bezel-section">
                        <div class="section-label">SECURITY TACTILE CONFIRMATION REQUIRED</div>
                        <div class="dangerous-overlay" style="position:relative; background:transparent;">
                            <div class="dangerous-command" style="margin-bottom:20px;">{cmd}</div>
                            <div class="slider-track" style="width:100%; max-width:400px;">
                                <input type="range" class="confirm-slider" min="0" max="100" value="0" 
                                    oninput="if(this.value == 100) {{ window.ipc.postMessage('prompt_submit:{cmd}'); window.ipc.postMessage('toggle_bezel'); }}"
                                    onchange="if(this.value < 100) {{ this.value = 0; }}">
                                <div class="slider-label">SLIDE TO CONFIRM</div>
                            </div>
                            <button class="bezel-btn danger" style="margin-top:20px;" onclick="window.ipc.postMessage('stage_command:')">ABORT ACTION</button>
                        </div>
                    </div>"#,
                    cmd = dangerous_cmd
                );
            }

            let level_specific_html = match level {
                HierarchyLevel::ApplicationFocus => {
                    let hub = &sector.hubs[viewport.hub_index];
                    let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];
                    
                    let priority = app.settings.get("priority").cloned().unwrap_or(5.0);
                    let gain = app.settings.get("gain").cloned().unwrap_or(75.0);
                    let sensitivity = app.settings.get("sensitivity").cloned().unwrap_or(40.0);
                    let portal_active_class = if sector.portal_active { "active" } else { "" };
                    let portal_label = if sector.portal_active { "DISABLE PORTAL" } else { "EXPORT PORTAL" };
                    let portal_url = sector.portal_url.as_ref().map(|u| format!("<div class='footer-stat'>URL: {}</div>", u)).unwrap_or_default();

                    format!(
                        r#"<div class="bezel-section">
                            <div class="section-label">PROCESS CALIBRATION</div>
                            <div class="bezel-group sliders">
                                 <div class="action-slider">
                                    <span>PRIORITY</span>
                                    <input type="range" min="1" max="10" step="1" value="{priority}" oninput="window.ipc.postMessage('update_setting:priority:' + this.value)">
                                 </div>
                                 <div class="action-slider">
                                    <span>GAIN</span>
                                    <input type="range" min="0" max="100" step="1" value="{gain}" oninput="window.ipc.postMessage('update_setting:gain:' + this.value)">
                                 </div>
                                 <div class="action-slider">
                                    <span>SENSITIVITY</span>
                                    <input type="range" min="0" max="100" step="1" value="{sensitivity}" oninput="window.ipc.postMessage('update_setting:sensitivity:' + this.value)">
                                 </div>
                            </div>
                        </div>
                        <div class="bezel-section">
                            <div class="section-label">PORTAL & CONNECTIVITY</div>
                            <div class="bezel-group">
                                <button class="bezel-btn {portal_active_class}" onclick="window.ipc.postMessage('toggle_portal')">{portal_label}</button>
                            </div>
                            {portal_url}
                        </div>
                        <div class="bezel-section">
                            <div class="section-label">NAVIGATION</div>
                            <div class="bezel-group">
                                <button class="bezel-btn" onclick="window.ipc.postMessage('zoom_out')">ZOOM OUT</button>
                                <button class="bezel-btn" onclick="window.ipc.postMessage('split_viewport')">SPLIT VIEW</button>
                                <button class="bezel-btn" onclick="window.ipc.postMessage('zoom_to:DetailInspector')">INSPECT</button>
                                <button class="bezel-btn danger" onclick="window.ipc.postMessage('kill_app')">CLOSE PROCESS</button>
                            </div>
                        </div>"#,
                        priority = priority,
                        gain = gain,
                        sensitivity = sensitivity,
                        portal_active_class = portal_active_class,
                        portal_label = portal_label,
                        portal_url = portal_url
                    )
                }
                _ => String::new(),
            };

            format!(
                r#"<div class="tactical-bezel expanded {priority_class}" style="--bezel-accent:{color};">
                    <div class="bezel-chips top">
                        <div class="chip"></div><div class="chip"></div><div class="chip"></div><div class="chip"></div>
                    </div>
                    <div class="bezel-top">
                        <div class="bezel-left">
                            <span class="bezel-title">{title} // EXPANDED TACTICAL VIEW</span>
                        </div>
                        <div class="bezel-right">
                            <div class="bezel-handle" onclick="window.ipc.postMessage('toggle_bezel')">
                                <div class="chevron"></div>
                            </div>
                        </div>
                    </div>
                    <div class="bezel-expanded-content">
                        {slider}
                        {level_specific}
                        <div class="bezel-section">
                            <div class="section-label">LEVEL METRICS</div>
                            <div class="bezel-group">
                                <div class="footer-stat">FPS: {fps:.1}</div>
                                <div class="footer-stat">LATENCY: 0.4ms</div>
                                <div class="footer-stat">SYNC: ACTIVE</div>
                            </div>
                        </div>
                        <div class="bezel-section">
                            <div class="section-label">QUICK ACTIONS</div>
                            <div class="bezel-group">
                                <button class="bezel-btn mini" onclick="window.ipc.postMessage('semantic_event:ToggleMiniMap')">MINIMAP</button>
                                <button class="bezel-btn mini" onclick="window.ipc.postMessage('semantic_event:ToggleComms')">COMMS</button>
                                <button class="bezel-btn mini danger" onclick="window.ipc.postMessage('tactical_reset')" {disabled_manage}>RESET</button>
                            </div>
                        </div>
                    </div>
                </div>"#,
                priority_class = priority_class,
                color = accent_color,
                title = title.to_uppercase(),
                slider = slider_html,
                level_specific = level_specific_html,
                fps = state.fps,
                disabled_manage = disabled_attr(can_manage_sector)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TosState, Viewport, HierarchyLevel, Sector, CommandHub, SemanticEvent};

    #[test]
    fn test_render_bezel_l3_collapsed() {
        let mut state = TosState::new_fresh();
        let sector = Sector {
            id: uuid::Uuid::new_v4(),
            name: "Test Sector".to_string(),
            color: "#ffffff".to_string(),
            hubs: vec![CommandHub {
                id: uuid::Uuid::new_v4(),
                mode: crate::CommandHubMode::Command,
                prompt: String::new(),
                applications: vec![crate::Application {
                    id: uuid::Uuid::new_v4(),
                    title: "Test App".to_string(),
                    app_class: "TestClass".to_string(),
                    is_minimized: false,
                    pid: None,
                    icon: None,
                    is_dummy: true,
                    settings: std::collections::HashMap::new(),
                    thumbnail: None,
                    decoration_policy: crate::DecorationPolicy::Native,
                    bezel_actions: vec![],
                }],
                active_app_index: Some(0),
                terminal_output: vec![],
                confirmation_required: None,
                current_directory: std::path::PathBuf::from("/"),
                show_hidden_files: false,
                selected_files: std::collections::HashSet::new(),
                context_menu: None,
                shell_listing: None,
                suggestions: vec![],
                output_mode_centered: false,
                left_region_visible: true,
            }],
            active_hub_index: 0,
            host: "LOCAL".to_string(),
            connection_type: crate::ConnectionType::Local,
            participants: vec![],
            portal_active: false,
            portal_url: None,
            description: "Test".to_string(),
            icon: "🧪".to_string(),
            sector_type_name: "test".to_string(),
        };
        state.sectors.push(sector);
        
        let viewport = Viewport {
            id: uuid::Uuid::new_v4(),
            sector_index: state.sectors.len() - 1,
            hub_index: 0,
            current_level: HierarchyLevel::ApplicationFocus,
            active_app_index: Some(0),
            bezel_expanded: false,
        };
        
        let html = render_bezel(&state, &viewport, HierarchyLevel::ApplicationFocus, BezelState::Collapsed);
        
        // Assertions for BZ-02
        assert!(html.contains("tactical-bezel"), "Should contain tactical-bezel class");
        assert!(html.contains("collapsed"), "Should contain collapsed class");
        assert!(html.contains("bezel-title"), "Should contain title class");
        assert!(html.contains("TEST APP // TESTCLASS"), "Should contain app title and class");
        assert!(html.contains("zoom_out"), "Should contain zoom out button/action");
    }

    #[test]
    fn test_render_bezel_l2_hub() {
        let mut state = TosState::new_fresh();
        let sector = Sector {
            id: uuid::Uuid::new_v4(),
            name: "Hub Sector".to_string(),
            color: "#ffcc00".to_string(),
            hubs: vec![CommandHub {
                id: uuid::Uuid::new_v4(),
                mode: crate::CommandHubMode::Command,
                prompt: String::new(),
                applications: vec![],
                active_app_index: None,
                terminal_output: Vec::new(),
                confirmation_required: None,
                current_directory: std::path::PathBuf::from("/"),
                show_hidden_files: false,
                selected_files: std::collections::HashSet::new(),
                context_menu: None,
                shell_listing: None,
                suggestions: vec![],
                output_mode_centered: false,
                left_region_visible: true,
            }],
            active_hub_index: 0,
            host: "LOCAL".to_string(),
            connection_type: crate::ConnectionType::Local,
            participants: vec![],
            portal_active: false,
            portal_url: None,
            description: "Test".to_string(),
            icon: "⚙️".to_string(),
            sector_type_name: "test".to_string(),
        };
        state.sectors.push(sector);
        
        let viewport = Viewport {
            id: uuid::Uuid::new_v4(),
            sector_index: state.sectors.len() - 1,
            hub_index: 0,
            current_level: HierarchyLevel::CommandHub,
            active_app_index: None,
            bezel_expanded: false,
        };
        
        let html = render_bezel(&state, &viewport, HierarchyLevel::CommandHub, BezelState::Collapsed);
        
        assert!(html.contains("tactical-bezel"), "Should contain tactical-bezel class");
        assert!(html.contains("toggle_output_mode") || html.contains("Perspective"), "Should contain output mode toggle");
        assert!(html.contains("toggle_left_region") || html.contains("Region"), "Should contain left region toggle");
    }

    #[test]
    fn test_render_bezel_l1_global() {
        let state = TosState::new_fresh();
        let viewport = Viewport {
            id: uuid::Uuid::new_v4(),
            sector_index: 0,
            hub_index: 0,
            current_level: HierarchyLevel::GlobalOverview,
            active_app_index: None,
            bezel_expanded: false,
        };
        
        let html = render_bezel(&state, &viewport, HierarchyLevel::GlobalOverview, BezelState::Collapsed);
        
        assert!(html.contains("tactical-bezel"), "Should contain tactical-bezel class");
        assert!(html.contains("settings") || html.contains("Gear"), "Should contain settings gear");
        assert!(html.contains("collaboration") || html.contains("Avatars"), "Should contain collaboration indicators");
    }

    #[test]
    fn test_hierarchy_round_trip() {
        let mut state = TosState::new_fresh();
        
        // Starts at GlobalOverview (Level 1)
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
        
        // Zoom In -> Level 2 (Command Hub)
        state.handle_semantic_event(SemanticEvent::ZoomIn);
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
        
        // Zoom In -> Level 3 (Application Focus)
        state.handle_semantic_event(SemanticEvent::ZoomIn);
        assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
        
        // Zoom Out -> Back to Level 2
        state.handle_semantic_event(SemanticEvent::ZoomOut);
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
        
        // Zoom Out -> Back to Level 1
        state.handle_semantic_event(SemanticEvent::ZoomOut);
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    }
}
