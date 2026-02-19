use crate::{TosState, Viewport, RenderMode, HierarchyLevel};
use super::ViewRenderer;

pub struct AppRenderer;

impl ViewRenderer for AppRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
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

        let portal_active_class = if sector.portal_active { "active" } else { "" };
        let portal_label = if sector.portal_active { "DISABLE PORTAL" } else { "EXPORT PORTAL" };
        
        // Portal approval dialog for 7.4 security
        let portal_approval_html = if state.is_portal_approval_pending() {
            let sector_name = state.get_approval_requested_sector_name().unwrap_or_default();
            format!(
                r#"<div class="portal-approval-overlay">
                    <div class="approval-dialog">
                        <div class="approval-title">TACTICAL APPROVAL REQUIRED</div>
                        <div class="approval-message">
                            Web portal access requested for sector:<br>
                            <strong>{}</strong>
                        </div>
                        <div class="approval-warning">
                            ⚠️ This will expose the sector to external web access.<br>
                            Tactile approval required on host machine.
                        </div>
                        <div class="approval-actions">
                            <button class="approval-btn approve" onclick="window.ipc.postMessage('approve_portal')">
                                ✓ APPROVE
                            </button>
                            <button class="approval-btn deny" onclick="window.ipc.postMessage('deny_portal')">
                                ✗ DENY
                            </button>
                        </div>
                    </div>
                </div>"#,
                sector_name
            )
        } else {
            String::new()
        };
        
        let portal_info_html = if sector.portal_active {
            format!(
                r#"<div class="bezel-status-panel">
                    <div class="status-label">WEB PORTAL ACTIVE</div>
                    <div class="status-value">{}</div>
                </div>"#,
                sector.portal_url.as_ref().unwrap_or(&"INITIALIZING...".to_string())
            )
        } else {
            String::new()
        };

        let deep_inspection_html = if state.security.deep_inspection_active {
            r#"<div class="bezel-btn warning mini" onclick="window.ipc.postMessage('security:disable_deep_inspection')" title="Disable Deep Inspection">🔓</div>"#.to_string()
        } else {
            String::new()
        };

        let mut module_content = String::new();
        for module in &state.modules {
            if let Some(content) = module.render_override(HierarchyLevel::ApplicationFocus) {
                module_content.push_str(&content);
            }
        }

            format!(
            r#"<div class="application-container render-{mode:?}">
                <div class="tactical-bezel {bezel_class}">
                    <div class="bezel-top">
                        <div class="bezel-back" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
                        {deep_inspection_html}
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
                            <div class="bezel-btn {portal_active_class}" onclick="window.ipc.postMessage('toggle_portal')">{portal_label}</div>
                            <div class="bezel-btn danger">CLOSE</div>
                        </div>
                        {portal_info_html}
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
                </div>
                {portal_approval_html}
                <div class="application-surface" onclick="window.ipc.postMessage('zoom_in')">
                    {app_content}
                </div>
            </div>"#,
            mode = mode,
            bezel_class = bezel_class,
            title = app.title.to_uppercase(),
            class = app.app_class.to_uppercase(),
            participants_html = participants_html,
            priority = app.settings.get("priority").cloned().unwrap_or(5.0),
            gain = app.settings.get("gain").cloned().unwrap_or(75.0),
            sensitivity = app.settings.get("sensitivity").cloned().unwrap_or(40.0),
            app_content = if app.app_class.contains("Shell") || app.app_class.contains("terminal") {
                let lines = hub.terminal_output.join("\n");
                format!(r#"<pre class="terminal-content">{}</pre>{}"#, lines, module_content)
            } else {
                format!(
                    r#"<div class="app-mock-content">
                        <div class="app-surface-header">
                            <div class="status-indicator active"></div>
                            <div class="app-id-badge">{title} // SEQ: {uuid_short}</div>
                            <div class="system-time">{time}</div>
                        </div>
                        <div class="app-surface-body">
                            <div class="data-stream">
                                {module_content}
                                <div class="mock-data-entry">INITIALIZING SUBSYSTEMS...</div>
                                <div class="mock-data-entry">ATTACHING TO KERNEL V{version}...</div>
                                <div class="mock-data-entry">ESTABLISHING DATA BUFFER...</div>
                                <div class="mock-data-block">
                                    <div class="graph-bar" style="width: 85%"></div>
                                    <div class="graph-bar" style="width: 62%"></div>
                                    <div class="graph-bar" style="width: 94%"></div>
                                </div>
                            </div>
                        </div>
                        <div class="app-surface-footer">
                            <div class="footer-stat">ID: {pid}</div>
                            <div class="footer-stat">VRAM: 128MB</div>
                            <div class="footer-stat">LOAD: {mem}</div>
                            <div class="bezel-btn mini">RECALIBRATE</div>
                        </div>
                    </div>"#,
                    title = app.title.to_uppercase(),
                    uuid_short = &app.id.to_string()[..8],
                    time = chrono::Local::now().format("%H:%M:%S").to_string(),
                    version = env!("CARGO_PKG_VERSION"),
                    module_content = module_content,
                    pid = app.pid.map(|p| p.to_string()).unwrap_or_else(|| "TOS-SYS".to_string()),
                    mem = if let Some(pid) = app.pid {
                        if let Ok(stats) = crate::system::proc::get_process_stats(pid) {
                            format!("{}MB", stats.memory_bytes / 1024 / 1024)
                        } else { "---MB".to_string() }
                    } else { "---MB".to_string() }
                )
            },
            portal_active_class = portal_active_class,
            portal_label = portal_label,
            portal_info_html = portal_info_html,
            portal_approval_html = portal_approval_html,
            deep_inspection_html = deep_inspection_html
        )

    }
}
