use crate::{TosState, Viewport, RenderMode, CommandHubMode, SectorType};
use super::ViewRenderer;

pub struct HubRenderer;

impl ViewRenderer for HubRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        
        let mode_type_class = match hub.mode {
            CommandHubMode::Command => "mode-command",
            CommandHubMode::Directory => "mode-directory",
            CommandHubMode::Activity => "mode-activity",
            CommandHubMode::Search => "mode-search",
            CommandHubMode::Ai => "mode-ai",
        };

        let perspective_class = if hub.output_mode_centered { "perspective-centered" } else { "" };
        let region_class = if hub.left_region_visible { "" } else { "left-region-hidden" };

        let mut html = format!(r#"<div class="command-hub {mode_type_class} {perspective_class} {region_class} render-{mode:?}">"#, mode = mode);

        let mut participants_html = String::new();
        for p in &sector.participants {
            let initials: String = p.name.split_whitespace().map(|s| s.chars().next().unwrap_or(' ')).collect();
            participants_html.push_str(&format!(
                r#"<div class="participant-avatar" style="background-color: {color}" title="{name} ({role})" onclick="window.ipc.postMessage('follow_participant:{id}')">{initials}</div>"#,
                color = p.color, name = p.name, role = p.role, initials = initials, id = p.id
            ));
        }

        let follow_indicator = if !state.collaboration_manager.following_modes.is_empty() {
             r#"<div class="follow-indicator active">FOLLOW MODE ACTIVE</div>"#
        } else {
             ""
        };

        // 1. Unified Tactical Header
        html.push_str(&format!(
            r#"<div class="tactical-header" style="--header-accent: {color};">
                <div class="header-left">
                    <button class="bezel-action-btn" onclick="window.ipc.postMessage('zoom_out')" title="Zoom Out">ZOOM OUT</button>
                    <button class="bezel-action-btn" onclick="window.ipc.postMessage('toggle_output_mode')" title="Standard / Centered Perspective">OUTPUT</button>
                    <button class="bezel-action-btn" onclick="window.ipc.postMessage('toggle_left_region')" title="Toggle Left Favourites">FAVS</button>
                </div>
                <div class="header-center">
                    <div class="three-way-toggle">
                        <div class="toggle-segment {cmd_active}" onclick="window.ipc.postMessage('set_mode:Command')">COMMAND</div>
                        <div class="toggle-segment {dir_active}" onclick="window.ipc.postMessage('set_mode:Directory')">DIRECTORY</div>
                        <div class="toggle-segment {act_active}" onclick="window.ipc.postMessage('set_mode:Activity')">ACTIVITY</div>
                    </div>
                </div>
                <div class="header-right">
                    <div class="hub-info-inline" style="display:flex; gap:15px; align-items:center;">
                        <div class="hub-metadata">
                            <span class="hub-sector-name" style="font-weight:800; color:{color};">{name}</span>
                            <span class="hub-host" style="font-size:0.7rem; opacity:0.6; margin-left:10px;">{host}</span>
                        </div>
                        {follow_indicator}
                        <div class="hub-participants" style="display:flex; gap:5px;">
                            {participants_html}
                        </div>
                        <div class="telemetry-item" style="display:flex; gap:15px; align-items:center;">
                            <button class="bezel-action-btn comms-toggle-btn" onclick="window.ipc.postMessage('toggle_comms')">COMMS</button>
                            <button class="bezel-action-btn minimap-toggle-btn" onclick="window.ipc.postMessage('semantic_event:ToggleMiniMap')">MAP</button>
                            <div class="bezel-clock" style="position: relative; top: 0; left: 0; margin-left: 5px;">{time}</div>
                        </div>
                    </div>
                </div>
            </div>"#,
            color = sector.color,
            name = sector.name.to_uppercase(),
            host = sector.host,
            participants_html = participants_html,
            follow_indicator = follow_indicator,
            cmd_active = if hub.mode == CommandHubMode::Command { "active" } else { "" },
            dir_active = if hub.mode == CommandHubMode::Directory { "active" } else { "" },
            act_active = if hub.mode == CommandHubMode::Activity { "active" } else { "" },
            time = state.get_system_time()
        ));

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
        // 2. Main Content Area (§2.3)
        // (Removed redundant hub-content push as it's already pushed at line 81)
        
        // Terminal Output Background (§2.4)
        let mut output_html = String::new();
        for line in &hub.terminal_output {
            let escaped = super::escape_html(line);
            output_html.push_str(&format!(r#"<div class="log-line">{}</div>"#, escaped));
        }

        html.push_str(&format!(
            r#"<div class="terminal-container background-layer">
                <div class="terminal-output" id="hub-term-{}">
                    {}
                </div>
            </div>"#,
            hub.id, output_html
        ));

        match hub.mode {
            CommandHubMode::Command => {
                let mut left_chips = String::new();
                let mut right_chips = String::new();
                
                // 1. Sector-specific favorites (Left Region)
                if let Some(sector_type) = state.sector_type_registry.get(&sector.sector_type_name) {
                    for fav_str in sector_type.command_favourites() {
                        let fav_str: String = fav_str;
                        if let Some((label, cmd)) = fav_str.split_once(':') {
                            left_chips.push_str(&format!(
                                r#"<button class="chip-btn" onclick="window.ipc.postMessage('stage_command:{} ')">{}</button>"#,
                                cmd.replace('\'', "\\'"),
                                label.to_uppercase()
                            ));
                        }
                    }
                }

                // 2. Shell provided suggestions (Right Region)
                for sug in &hub.suggestions {
                    right_chips.push_str(&format!(
                        r#"<button class="chip-btn priority-high" onclick="window.ipc.postMessage('stage_command:{}')" title="{}">{}</button>"#,
                        sug.command.replace('\'', "\\'"),
                        sug.description.replace('\'', "\\'"),
                        sug.text
                    ));
                }
                
                // 3. System defaults if empty (Left Region)
                if left_chips.is_empty() {
                    let defaults = [
                        ("LS", "ls -F"),
                        ("GIT STATUS", "git status"),
                        ("PROCESSES", "ps aux"),
                        ("RELOAD", "source ~/.config/fish/config.fish"),
                    ];
                    for (label, cmd) in defaults {
                        left_chips.push_str(&format!(
                            r#"<button class="chip-btn" onclick="window.ipc.postMessage('stage_command:{} ')">{}</button>"#,
                            cmd, label
                        ));
                    }
                }

                html.push_str(&format!(r#"<div class="command-view foreground-layer">
                    <div class="left-chip-region">
                        <div class="chip-region-title" style="font-size:0.7em;color:var(--lcars-orange);margin-bottom:10px;">FAVORITES</div>
                        {}
                    </div>
                    <div class="terminal-spacer" style="flex:1; pointer-events:none;"></div>
                    <div class="right-chip-region">
                        <div class="chip-region-title" style="font-size:0.7em;color:var(--lcars-orange);margin-bottom:10px;">SUGGESTIONS</div>
                        {}
                    </div>
                </div>"#, left_chips, right_chips));
            }
            CommandHubMode::Directory => {
                let cwd = &hub.current_directory;
                let _cwd_display = cwd.display().to_string().to_uppercase();

                html.push_str(r#"<div class="directory-view foreground-layer">
                    <div class="path-bar breadcrumbs">"#);
                
                let _path_str = hub.current_directory.to_string_lossy();
                let mut current_path = std::path::PathBuf::new();
                
                // Add Root /
                html.push_str(r#"<span class="breadcrumb" onclick="window.ipc.postMessage('dir_navigate:/')">ROOT</span>"#);
                
                for component in hub.current_directory.components() {
                    if let std::path::Component::Normal(name) = component {
                        let name_str = name.to_string_lossy();
                        current_path.push(name);
                        let full_path = current_path.to_string_lossy().replace('\'', "\\'");
                        html.push_str(&format!(
                            r#"<span class="path-sep">/</span><span class="breadcrumb" onclick="window.ipc.postMessage('dir_navigate:{}')">{}</span>"#,
                            full_path, name_str.to_uppercase()
                        ));
                    }
                }
                html.push_str("</div>");

                // Action Bar (§3.2 Action toolbar)
                html.push_str(r#"<div class="directory-actions">
                    <button class="bezel-btn" onclick="window.ipc.postMessage('stage_command:mkdir ')">NEW FOLDER</button>
                    <button class="bezel-btn" onclick="window.ipc.postMessage('dir_action_copy')">COPY</button>
                    <button class="bezel-btn" onclick="window.ipc.postMessage('dir_action_paste')">PASTE</button>
                    <button class="bezel-btn" onclick="window.ipc.postMessage('stage_command:mv ')">RENAME</button>
                    <button class="bezel-btn" onclick="window.ipc.postMessage('dir_toggle_hidden')">TOGGLE HIDDEN</button>
                    <button class="bezel-btn danger" onclick="window.ipc.postMessage('stage_command:rm ')">DELETE</button>
                    <button class="bezel-btn" onclick="window.ipc.postMessage('dir_navigate:.')">REFRESH</button>
                </div>"#);

                html.push_str(r#"<div class="file-grid">"#);

                // Always show parent directory entry
                html.push_str(r#"<div class="file-item staging-item" onclick="window.ipc.postMessage('dir_navigate:..')">
                    <span class="file-icon folder"></span> ..
                </div>"#);

                // Check if we have a shell-provided listing for this directory
                let use_shell = sector.connection_type != crate::ConnectionType::Local || hub.shell_listing.is_some();
                
                if use_shell {
                    if let Some(listing) = &hub.shell_listing {
                        for entry in &listing.entries {
                            if !hub.show_hidden_files && entry.is_hidden {
                                continue;
                            }
                            
                            let is_dir = entry.entry_type == crate::EntryType::Directory;
                            let is_selected = hub.selected_files.contains(&entry.name);
                            let selected_class = if is_selected { "selected" } else { "" };
                            let escaped_name = entry.name.replace('\'', "\\'");
                            let display_name = entry.name.to_uppercase();
                            
                            if is_dir {
                                html.push_str(&format!(
                                    r#"<div class="file-item staging-item {selected_class}" 
                                        onclick="window.ipc.postMessage('dir_navigate:{escaped}')"
                                        oncontextmenu="event.preventDefault(); window.ipc.postMessage('dir_context:{escaped};' + event.clientX + ';' + event.clientY)">
                                        <div class="file-selector" onclick="event.stopPropagation(); window.ipc.postMessage('dir_toggle_select:{escaped}')"></div>
                                        <span class="file-icon folder"></span> {display}/
                                    </div>"#,
                                    escaped = escaped_name,
                                    display = display_name,
                                    selected_class = selected_class,
                                ));
                            } else {
                                let size_str = if entry.size > 1024 * 1024 {
                                    format!("{:.1}MB", entry.size as f32 / 1024.0 / 1024.0)
                                } else {
                                    format!("{}KB", entry.size / 1024)
                                };
                                
                                html.push_str(&format!(
                                    r#"<div class="file-item staging-item {selected_class}" 
                                        onclick="window.ipc.postMessage('dir_pick_file:{escaped}')"
                                        oncontextmenu="event.preventDefault(); window.ipc.postMessage('dir_context:{escaped};' + event.clientX + ';' + event.clientY)">
                                        <div class="file-selector" onclick="event.stopPropagation(); window.ipc.postMessage('dir_toggle_select:{escaped}')"></div>
                                        <span class="file-icon file"></span>
                                        <div class="file-info">
                                            <div class="file-name">{display}</div>
                                            <div class="file-meta">{size}</div>
                                        </div>
                                    </div>"#,
                                    escaped = escaped_name,
                                    display = display_name,
                                    size = size_str,
                                    selected_class = selected_class,
                                ));
                            }
                        }
                    } else {
                        html.push_str(r#"<div class="file-item status-item">WAITING FOR SHELL...</div>"#);
                    }
                } else {
                // Read actual filesystem
                match std::fs::read_dir(cwd) {
                    Ok(entries) => {
                        let mut dirs: Vec<(String, std::fs::Metadata)> = Vec::new();
                        let mut files: Vec<(String, std::fs::Metadata)> = Vec::new();

                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();

                            // Skip hidden files unless enabled
                            if !hub.show_hidden_files && name.starts_with('.') {
                                continue;
                            }

                            if let Ok(meta) = entry.metadata() {
                                if meta.is_dir() {
                                    dirs.push((name, meta));
                                } else {
                                    files.push((name, meta));
                                }
                            }
                        }

                        // Sort alphabetically (case-insensitive)
                        dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
                        files.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

                        // Render directories first
                        for (name, _meta) in &dirs {
                            let display_name = name.to_uppercase();
                            let escaped_name = name.replace('\'', "\\'");
                            let is_selected = hub.selected_files.contains(name);
                            let selected_class = if is_selected { "selected" } else { "" };
                            
                            html.push_str(&format!(
                                r#"<div class="file-item staging-item {selected_class}" 
                                    onclick="window.ipc.postMessage('dir_navigate:{escaped}')"
                                    oncontextmenu="event.preventDefault(); window.ipc.postMessage('dir_context:{escaped};' + event.clientX + ';' + event.clientY)">
                                    <div class="file-selector" onclick="event.stopPropagation(); window.ipc.postMessage('dir_toggle_select:{escaped}')"></div>
                                    <span class="file-icon folder"></span> {display}/
                                </div>"#,
                                escaped = escaped_name,
                                display = display_name,
                                selected_class = selected_class,
                            ));
                        }

                        // Render files
                        for (name, meta) in &files {
                            let display_name = name.to_uppercase();
                            let escaped_name = name.replace('\'', "\\'");
                            let size = meta.len();
                            let size_str = if size > 1024 * 1024 {
                                format!("{:.1}MB", size as f32 / (1024.1 * 1024.1))
                            } else {
                                format!("{}KB", size / 1024)
                            };
                            let ext = std::path::Path::new(name)
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("FILE")
                                .to_uppercase();

                            let is_selected = hub.selected_files.contains(name);
                            let selected_class = if is_selected { "selected" } else { "" };

                            html.push_str(&format!(
                                r#"<div class="file-item staging-item {selected_class}" 
                                    onclick="window.ipc.postMessage('dir_pick_file:{escaped}')"
                                    oncontextmenu="event.preventDefault(); window.ipc.postMessage('dir_context:{escaped};' + event.clientX + ';' + event.clientY)">
                                    <div class="file-selector" onclick="event.stopPropagation(); window.ipc.postMessage('dir_toggle_select:{escaped}')"></div>
                                    <span class="file-icon {ext_class}"></span>
                                    <div class="file-info">
                                        <div class="file-name">{display}</div>
                                        <div class="file-meta">{size} // {ext}</div>
                                    </div>
                                </div>"#,
                                escaped = escaped_name,
                                display = display_name,
                                size = size_str,
                                ext = ext,
                                ext_class = ext.to_lowercase(),
                                selected_class = selected_class,
                            ));
                        }

                        // Show hidden toggle
                        let toggle_label = if hub.show_hidden_files { "HIDE HIDDEN" } else { "SHOW HIDDEN" };
                        html.push_str(&format!(
                            r#"<div class="file-item action-item" onclick="window.ipc.postMessage('dir_toggle_hidden')">
                                <span class="file-icon"></span> {} FILES
                            </div>"#,
                            toggle_label
                        ));
                    }
                    Err(e) => {
                        html.push_str(&format!(
                            r#"<div class="file-item error-item">
                                <span class="file-icon"></span> ACCESS DENIED: {}
                            </div>"#,
                            e.to_string().to_uppercase()
                        ));
                    }
                }
            }

                        html.push_str("</div>");

                        // Render action toolbar if multiple files are selected
                        if !hub.selected_files.is_empty() {
                            let count = hub.selected_files.len();
                            html.push_str(&format!(
                                r#"<div class="action-toolbar">
                                    <div class="toolbar-label">{} FILES SELECTED</div>
                                    <div class="toolbar-actions">
                                        <button class="bezel-btn" onclick="window.ipc.postMessage('dir_batch_copy')">REPLICATE</button>
                                        <button class="bezel-btn danger" onclick="window.ipc.postMessage('dir_batch_delete')">PURGE</button>
                                        <button class="bezel-btn" onclick="window.ipc.postMessage('dir_clear_select')">CLEAR</button>
                                    </div>
                                </div>"#,
                                count
                            ));
                        }

                        // Render context menu if active
                        if let Some(menu) = &hub.context_menu {
                            let mut actions_html = String::new();
                            for action in &menu.actions {
                                let cmd = match action.as_str() {
                                    "OPEN" => format!("view {}", menu.target),
                                    "COPY" => format!("cp {} ", menu.target),
                                    "DELETE" => format!("rm {}", menu.target),
                                    "RENAME" => format!("mv {} ", menu.target),
                                    _ => action.clone(),
                                };
                                actions_html.push_str(&format!(
                                    r#"<div class="menu-item" onclick="window.ipc.postMessage('stage_command:{}'); window.ipc.postMessage('dir_close_context')">{}</div>"#,
                                    cmd.replace('\'', "\\'"), action
                                ));
                            }

                            html.push_str(&format!(
                                r#"<div class="context-menu-overlay" onclick="window.ipc.postMessage('dir_close_context')">
                                    <div class="context-menu" style="left: {}px; top: {}px;" onclick="event.stopPropagation()">
                                        <div class="menu-header">{}</div>
                                        {}
                                    </div>
                                </div>"#,
                                menu.x, menu.y, menu.target.to_uppercase(), actions_html
                            ));
                        }

                        html.push_str("</div>");
                    }
            CommandHubMode::Activity => {
                let mut apps_html = String::new();
                for app in &hub.applications {
                    let mut cpu_str = "CPU: ---".to_string();
                    let mut mem_str = "MEM: ---".to_string();
                    let mut pid_str = "PID: ---".to_string();

                    if let Some(pid) = app.pid {
                        pid_str = format!("PID: {}", pid);
                        if let Ok(stats) = crate::system::proc::get_process_stats(pid) {
                            cpu_str = format!("CPU: {:.1}%", stats.cpu_usage);
                            mem_str = format!("MEM: {}MB", stats.memory_bytes / 1024 / 1024);
                        }
                    } else if app.is_dummy {
                        cpu_str = "CPU: 0.0%".to_string();
                        mem_str = "MEM: <1MB".to_string();
                        pid_str = "PID: [TOS]".to_string();
                    }

                    let icon = app.icon.as_deref().unwrap_or("⚙️");

                    let is_selected = hub.selected_files.contains(&app.id.to_string());
                    let selected_class = if is_selected { "selected" } else { "" };

                    apps_html.push_str(&format!(
                        r#"<div class="app-tile staging-item {selected_class}">
                            <div class="file-selector" onclick="event.stopPropagation(); window.ipc.postMessage('app_toggle_select:{id}')"></div>
                            <div class="app-tile-header" onclick="window.ipc.postMessage('stage_command:focus {title}')">
                                <div class="app-tile-icon">{icon}</div>
                                <div class="app-tile-info">
                                    <div class="app-title">{title}</div>
                                    <div class="app-class">{class}</div>
                                    <div class="app-pid" style="font-size: 0.7em; color: var(--tos-gold); opacity: 0.7;">{pid}</div>
                                </div>
                            </div>
                            <div class="app-tile-stats">
                                <div class="stat">{cpu}</div>
                                <div class="stat">{mem}</div>
                            </div>
                            <div class="app-tile-actions">
                                <button class="tile-btn" onclick="window.ipc.postMessage('signal_app:{id};INT')">SIGINT</button>
                                <button class="tile-btn danger" onclick="window.ipc.postMessage('kill_app:{id}')">KILL</button>
                            </div>
                        </div>"#,
                        title = app.title.to_uppercase(),
                        class = app.app_class.to_uppercase(),
                        icon = icon,
                        pid = pid_str,
                        cpu = cpu_str,
                        mem = mem_str,
                        id = app.id,
                        selected_class = selected_class
                    ));
                }
                // Module Data Feeds Implementation
                let mut modules_html = String::new();
                for info in state.module_registry.modules.values() {
                    if let Some(ref module) = info.module {
                        if let Some(overlay) = module.render_override(crate::HierarchyLevel::CommandHub) {
                            modules_html.push_str(&overlay);
                        }
                    }
                }

                // Calculate toolbar HTML
                let toolbar_html = if !hub.selected_files.is_empty() {
                    format!(r#"<div class="action-toolbar">
                        <div class="toolbar-label">{} APPS SELECTED</div>
                        <div class="toolbar-actions">
                            <button class="bezel-btn danger" onclick="window.ipc.postMessage('app_batch_kill')">KILL</button>
                            <button class="bezel-btn" onclick="window.ipc.postMessage('app_batch_signal:INT')">SIGINT</button>
                            <button class="bezel-btn" onclick="window.ipc.postMessage('dir_clear_select')">CLEAR</button>
                        </div>
                    </div>"#, hub.selected_files.len())
                } else {
                    String::new()
                };

                html.push_str(&format!(
                    r#"<div class="activity-view foreground-layer">
                        <div class="activity-section">
                            <div class="section-title">ACTIVE PROCESSES</div>
                            {toolbar_html}
                            <div class="activity-grid">
                                {apps_html}
                                <div class="app-tile add-tile" onclick="window.ipc.postMessage('spawn_app')">
                                    <span>+ NEW PROCESS</span>
                                </div>
                            </div>
                        </div>
                        <div class="activity-section">
                            <div class="section-title">MODULE DATA FEEDS</div>
                            <div class="module-data-grid">
                                {modules_html}
                            </div>
                        </div>
                        <div class="activity-section">
                            <div class="section-title">SECTOR TEMPLATES</div>
                            <div class="template-registry">
                                {templates_html}
                                <div class="template-item action-item" onclick="window.ipc.postMessage('save_template:Current-Sector')">
                                    <div class="template-name">+ SAVE CURRENT AS TEMPLATE</div>
                                </div>
                            </div>
                        </div>
                    </div>"#,
                    toolbar_html = toolbar_html,
                    apps_html = apps_html,
                    modules_html = modules_html,
                    templates_html = {
                        let mut html = String::new();
                        for template in state.get_available_templates() {
                            html.push_str(&format!(
                                r#"<div class="template-item" onclick="window.ipc.postMessage('load_template:{}')">
                                    <div class="template-name">{}</div>
                                    <div class="template-meta">{}</div>
                                </div>"#,
                                template.name, template.name, template.description
                            ));
                        }
                        if html.is_empty() {
                            r#"<div class="template-item empty">NO TEMPLATES FOUND</div>"#.to_string()
                        } else {
                            html
                        }
                    }
                ));
            }
            CommandHubMode::Ai => {
                let mut history_html = String::new();
                for msg in &state.ai_manager.history {
                    let role_class = match msg.role.as_str() {
                        "assistant" => "ai-msg-assistant",
                        "user" => "ai-msg-user",
                        _ => "ai-msg-system",
                    };
                    history_html.push_str(&format!(
                        r#"<div class="ai-msg {role_class}">
                            <div class="ai-role">{}</div>
                            <div class="ai-content">{}</div>
                        </div>"#,
                        msg.role.to_uppercase(),
                        msg.content
                    ));
                }
                
                let status_html = if state.ai_manager.is_generating {
                    r#"<div class="ai-status">PROCESSING... <span class="scanning"></span></div>"#
                } else {
                    ""
                };

                html.push_str(&format!(
                    r#"<div class="ai-view">
                        <div class="ai-history">
                            {}
                            {}
                        </div>
                    </div>"#,
                    history_html,
                    status_html
                ));
            }
            CommandHubMode::Search => {
                let mut results_html = String::new();
                for res in &state.search_manager.results {
                    let priority_class = match res.priority_score {
                        0..=2 => "priority-low",
                        3..=6 => "priority-med",
                        _ => "priority-high",
                    };
                    
                    let chips_count = (res.priority_score as f32 / 2.0).ceil() as usize;
                    let mut chips_html = String::new();
                    for _ in 0..chips_count {
                        chips_html.push_str(r#"<div class="priority-chip"></div>"#);
                    }

                    results_html.push_str(&format!(
                        r#"<div class="search-result staging-item {}" onclick="window.ipc.postMessage('search_select:{}')">
                            <div class="priority-indicator">
                                {}
                            </div>
                            <div class="search-meta">
                                <span class="search-domain">{:?}</span>
                                <span class="search-relevance">{}%</span>
                            </div>
                            <div class="search-title">{}</div>
                            <div class="search-desc">{}</div>
                        </div>"#,
                        priority_class,
                        res.id,
                        chips_html,
                        res.domain,
                        (res.relevance * 100.0) as u32,
                        res.title,
                        res.description
                    ));
                }

                if results_html.is_empty() {
                    results_html = r#"<div class="search-empty">ENTER QUERY TO SEARCH DOMAINS...</div>"#.to_string();
                }

                html.push_str(&format!(
                    r#"<div class="search-view">
                        <div class="search-grid">
                            {}
                        </div>
                    </div>"#,
                    results_html
                ));
            }
        }
        html.push_str("</div>");

        html.push_str(&format!(
            r#"<div class="unified-prompt">
                <div class="left-section">
                    <div class="three-way-toggle">
                        <div class="toggle-segment {cmd_active}" onclick="window.ipc.postMessage('set_mode:Command')">CMD</div>
                        <div class="toggle-segment {search_active}" onclick="window.ipc.postMessage('set_mode:Search')">SEARCH</div>
                        <div class="toggle-segment {ai_active}" onclick="window.ipc.postMessage('set_mode:Ai')">AI</div>
                    </div>
                </div>
                <div class="center-section">
                    <div class="prompt-container">
                        <div class="prompt-prefix">TOS@{} ></div>
                        <input type="text" id="terminal-input" value="{}" 
                            onkeydown="window.handlePromptKey(event)" 
                            oninput="window.handlePromptInput(event)"
                            autofocus>
                    </div>
                </div>
                <div class="right-section">
                    <div class="prompt-controls">
                        <div class="voice-trigger" style="font-size:1.5rem;cursor:pointer;" onclick="window.ipc.postMessage('semantic_event:VoiceCommandStart')">
                            {voice_indicator}
                        </div>
                        {ai_control}
                        <div class="stop-btn" onclick="window.ipc.postMessage('semantic_event:StopOperation')" title="STOP (Ctrl+Shift+C)">⏹️</div>
                    </div>
                </div>
            </div>"#,
            sector.name.to_uppercase(), hub.prompt,
            voice_indicator = state.voice.render_indicator(),
            ai_control = if hub.mode == CommandHubMode::Ai { 
                r#"<div class="ai-submit-btn" onclick="window.ipc.postMessage('semantic_event:AiSubmit')">↑</div>"# 
            } else { "" },
            cmd_active = if matches!(hub.mode, CommandHubMode::Command | CommandHubMode::Directory | CommandHubMode::Activity) { "active" } else { "" },
            search_active = if hub.mode == CommandHubMode::Search { "active" } else { "" },
            ai_active = if hub.mode == CommandHubMode::Ai { "active" } else { "" }
        ));

        html.push_str("</div>");
        
        if state.voice.is_listening() {
            html.push_str(&format!(
                r#"<div class="voice-overlay">
                    {}
                </div>"#,
                state.voice.render_help()
            ));
        }

        html
    }
}
