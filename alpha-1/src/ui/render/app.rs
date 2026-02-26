use crate::{TosState, Viewport, RenderMode, HierarchyLevel};
use super::ViewRenderer;

pub struct AppRenderer;

impl ViewRenderer for AppRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];
        let mut module_content = String::new();
        for module in &state.modules {
            if let Some(content) = module.render_override(HierarchyLevel::ApplicationFocus) {
                module_content.push_str(&content);
            }
        }

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

        format!(
            r#"<div class="application-container render-{mode:?}">
                {portal_approval}
                <div class="application-surface" onclick="window.ipc.postMessage('zoom_in')">
                    {app_content}
                </div>
            </div>"#,
            mode = mode,
            portal_approval = portal_approval_html,
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
                                <div class="data-stream-title">DATAFEED // REAL-TIME ANALYTICS</div>
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
                            <div class="bezel-btn mini" onclick="window.ipc.postMessage('tactical_reset')">RECALIBRATE</div>
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
            }
        )

    }
}
