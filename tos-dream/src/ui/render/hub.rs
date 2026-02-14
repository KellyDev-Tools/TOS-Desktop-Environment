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
}
