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
                let cwd_display = cwd.display().to_string().to_uppercase();

                html.push_str(&format!(r#"<div class="directory-view">
                    <div class="path-bar">{}</div>
                    <div class="file-grid">"#, cwd_display));

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
                            html.push_str(&format!(
                                r#"<div class="file-item staging-item" onclick="window.ipc.postMessage('dir_navigate:{escaped}')">
                                    <span class="file-icon folder"></span> {display}/
                                </div>"#,
                                escaped = escaped_name,
                                display = display_name,
                            ));
                        }

                        // Render files
                        for (name, meta) in &files {
                            let display_name = name.to_uppercase();
                            let escaped_name = name.replace('\'', "\\'");
                            let size = meta.len();
                            let size_str = if size < 1024 {
                                format!("{}B", size)
                            } else if size < 1024 * 1024 {
                                format!("{:.1}KB", size as f64 / 1024.0)
                            } else {
                                format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
                            };

                            // Determine file type from extension
                            let ext = std::path::Path::new(&name)
                                .extension()
                                .map(|e| e.to_string_lossy().to_uppercase())
                                .unwrap_or_else(|| "FILE".to_string());

                            html.push_str(&format!(
                                r#"<div class="file-item staging-item" onclick="window.ipc.postMessage('stage_command:view {escaped}')">
                                    <span class="file-icon file"></span> {display}
                                    <span class="file-meta">{size} // {ext}</span>
                                </div>"#,
                                escaped = escaped_name,
                                display = display_name,
                                size = size_str,
                                ext = ext,
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

                html.push_str("</div></div>");
            }
            CommandHubMode::Activity => {
                let mut apps_html = String::new();
                for app in &hub.applications {
                    apps_html.push_str(&format!(
                        r#"<div class="app-tile staging-item">
                            <div class="app-tile-header" onclick="window.ipc.postMessage('stage_command:focus {title}')">
                                <div class="app-tile-icon"></div>
                                <div class="app-tile-info">
                                    <div class="app-title">{title}</div>
                                    <div class="app-class">{class}</div>
                                </div>
                            </div>
                            <div class="app-tile-stats">
                                <div class="stat">CPU: 2.1%</div>
                                <div class="stat">MEM: 82MB</div>
                            </div>
                            <div class="app-tile-actions">
                                <button class="tile-btn danger" onclick="window.ipc.postMessage('kill_app:{id}')">KILL</button>
                            </div>
                        </div>"#,
                        title = app.title.to_uppercase(),
                        class = app.app_class.to_uppercase(),
                        id = app.id
                    ));
                }
                html.push_str(&format!(
                    r#"<div class="activity-view">
                        <div class="activity-section">
                            <div class="section-title">ACTIVE PROCESSES</div>
                            <div class="activity-grid">
                                {apps_html}
                                <div class="app-tile add-tile" onclick="window.ipc.postMessage('stage_command:spawn ')">
                                    <span>+ NEW PROCESS</span>
                                </div>
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
