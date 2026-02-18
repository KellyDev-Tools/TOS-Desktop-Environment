use crate::{TosState, Viewport, RenderMode, CommandHubMode};
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
        };

        let mut html = format!(r#"<div class="command-hub {mode_type_class} render-{mode:?}">"#, mode = mode);

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
                let cwd = &hub.current_directory;
                let _cwd_display = cwd.display().to_string().to_uppercase();

                html.push_str(r#"<div class="directory-view">
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
                    <button class="bezel-btn" onclick="window.ipc.postMessage('dir_navigate:.')">REFRESH</button>
                    <button class="bezel-btn danger" onclick="window.ipc.postMessage('stage_command:rm ')">DELETE</button>
                </div>"#);

                html.push_str(r#"<div class="file-grid">"#);

                // Always show parent directory entry
                html.push_str(r#"<div class="file-item staging-item" onclick="window.ipc.postMessage('dir_navigate:..')">
                    <span class="file-icon folder"></span> ..
                </div>"#);

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
                                    onclick="window.ipc.postMessage('stage_command:view {escaped}')"
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

                    apps_html.push_str(&format!(
                        r#"<div class="app-tile staging-item">
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
                        id = app.id
                    ));
                }
                // New: Module Data Feeds (Phase 16)
                let mut modules_html = String::new();
                for info in state.module_registry.modules.values() {
                    if let Some(ref module) = info.module {
                        if let Some(overlay) = module.render_override(crate::HierarchyLevel::CommandHub) {
                            modules_html.push_str(&overlay);
                        }
                    }
                }

                html.push_str(&format!(
                    r#"<div class="activity-view">
                        <div class="activity-section">
                            <div class="section-title">ACTIVE PROCESSES</div>
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
                                <div class="template-item" onclick="window.ipc.postMessage('load_template:Dev-Grid')">
                                    <div class="template-name">DEV-GRID</div>
                                    <div class="template-meta">3 HUBS // 5 APPS</div>
                                </div>
                                <div class="template-item" onclick="window.ipc.postMessage('load_template:Science-Lab')">
                                    <div class="template-name">SCIENCE-LAB</div>
                                    <div class="template-meta">1 HUB // 2 APPS</div>
                                </div>
                                <div class="template-item action-item" onclick="window.ipc.postMessage('save_template:Current-Sector')">
                                    <div class="template-name">+ SAVE CURRENT AS TEMPLATE</div>
                                </div>
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
}
