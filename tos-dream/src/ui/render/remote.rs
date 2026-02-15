use crate::{TosState, Viewport, RenderMode};
use super::ViewRenderer;

pub struct RemoteDesktopRenderer;

impl ViewRenderer for RemoteDesktopRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];
        
        let bezel_class = if viewport.bezel_expanded { "expanded" } else { "collapsed" };
        let protocol_str = match sector.connection_type {
            crate::ConnectionType::Local => "LOCAL",
            crate::ConnectionType::TOSNative => "TOS-NATIVE",
            crate::ConnectionType::SSH => "SSH-PTY",
            crate::ConnectionType::HTTP => "HTTP-FALLBACK",
        };

        let participants_html = sector.participants.iter().map(|p| {
            format!(r#"<div class="participant-avatar" style="background: {};" title="{} ({})"></div>"#, 
                p.color, p.name, p.role)
        }).collect::<String>();

        format!(
            r#"<div class="application-container remote-desktop-view render-{mode:?}">
                <div class="tactical-bezel {bezel_class}">
                    <div class="bezel-top">
                        <div class="bezel-title">{title} // {host} // {protocol}</div>
                        <div class="bezel-participants">{participants_html}</div>
                        <div class="bezel-handle" onclick="window.ipc.postMessage('toggle_bezel')">
                            <div class="chevron"></div>
                        </div>
                    </div>
                    <div class="bezel-expanded-content">
                        <div class="bezel-group">
                            <div class="bezel-btn" onclick="window.ipc.postMessage('zoom_out')">ZOOM OUT</div>
                            <div class="bezel-btn" onclick="window.ipc.postMessage('split_viewport')">SPLIT VIEW</div>
                        </div>
                        <div class="bezel-group sliders">
                             <div class="action-slider">
                                <span>STREAM QUALITY</span>
                                <input type="range" min="1" max="100" value="85">
                             </div>
                             <div class="action-slider">
                                <span>LATENCY MITIGATION</span>
                                <input type="range" min="1" max="100" value="40">
                             </div>
                        </div>
                        <div class="bezel-group">
                            <div class="bezel-btn" onclick="window.ipc.postMessage('invite_participant:Operator')">+ INVITE OPERATOR</div>
                            <div class="bezel-btn" onclick="window.ipc.postMessage('invite_participant:Viewer')">+ INVITE VIEWER</div>
                            <div class="bezel-btn danger">TERMINATE LINK</div>
                        </div>
                    </div>
                </div>
                <div class="application-surface remote-stream-surface">
                    <div class="remote-desktop-placeholder">
                        <div class="status-indicator ripple"></div>
                        <div class="link-label">RECEIVING STREAM: {host}</div>
                        <div class="link-sub">PROTOCOL: {protocol} // SYNC-ACTIVE</div>
                        
                        <div class="desktop-mock-ui">
                            <div class="mock-window"></div>
                            <div class="mock-window"></div>
                        </div>
                    </div>
                </div>
            </div>"#,
            mode = mode,
            bezel_class = bezel_class,
            title = app.title.to_uppercase(),
            host = sector.host,
            protocol = protocol_str,
            participants_html = participants_html
        )
    }
}
