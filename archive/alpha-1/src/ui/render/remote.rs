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

        // Fetch actual frame buffer if available (Section 7.3)
        let stream_content = if let Some(fb) = state.remote_manager.get_frame_buffer(sector.id) {
            format!(
                r#"<div class="remote-frame-buffer">
                    <img src="data:image/png;base64,{}" style="width: 100%; height: 100%; object-fit: contain;" />
                </div>"#,
                fb.to_base64()
            )
        } else {
            format!(
                r#"<div class="remote-desktop-placeholder">
                    <div class="status-indicator ripple"></div>
                    <div class="link-label">RECEIVING STREAM: {}</div>
                    <div class="link-sub">PROTOCOL: {} // SYNC-ACTIVE</div>
                    
                    <div class="desktop-mock-ui">
                        <div class="mock-window"></div>
                        <div class="mock-window"></div>
                    </div>
                </div>"#,
                sector.host, protocol_str
            )
        };

        let (latency, quality) = if let Some(conn) = state.remote_manager.active_connections.get(&sector.id) {
            (conn.latency_ms, conn.stream_quality)
        } else {
            (0, 0)
        };

        format!(
            r#"<div class="application-container remote-desktop-view render-{mode:?}">
                <div class="tactical-bezel {bezel_class}">
                    <div class="bezel-top">
                        <div class="bezel-title">{title} // {host} // {protocol} // {latency}MS // Q:{quality}%</div>
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
                                <input type="range" min="1" max="100" value="{quality}" onchange="window.ipc.postMessage('set_stream_quality:' + this.value)">
                             </div>
                             <div class="action-slider">
                                <span>LATENCY MITIGATION</span>
                                <input type="range" min="1" max="100" value="40">
                             </div>
                        </div>
                        <div class="bezel-group">
                            <div class="bezel-btn" onclick="window.ipc.postMessage('invite_participant:Operator')">+ INVITE OPERATOR</div>
                            <div class="bezel-btn" onclick="window.ipc.postMessage('invite_participant:Viewer')">+ INVITE VIEWER</div>
                            <div class="bezel-btn danger" onclick="window.ipc.postMessage('terminate_remote_link')">TERMINATE LINK</div>
                        </div>
                    </div>
                </div>
                <div class="application-surface remote-stream-surface">
                    {stream_content}
                </div>
            </div>"#,
            mode = mode,
            bezel_class = bezel_class,
            title = app.title.to_uppercase(),
            host = sector.host,
            protocol = protocol_str,
            latency = latency,
            quality = quality,
            participants_html = participants_html,
            stream_content = stream_content
        )
    }
}
