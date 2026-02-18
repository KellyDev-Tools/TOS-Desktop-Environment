use crate::{TosState, Viewport, RenderMode, ConnectionType};
use super::ViewRenderer;

pub struct GlobalRenderer;

impl ViewRenderer for GlobalRenderer {
    fn render(&self, state: &TosState, _viewport: &Viewport, mode: RenderMode) -> String {
        let mut html = String::new();

        // 1. Dashboard Header
        html.push_str(r#"<div class="dashboard-header">
            <h1 class="dashboard-title">TOS COMMAND CENTER</h1>
        </div>"#);

        // 2. Tactical Mini-Map
        html.push_str(r##"<div class="tactical-minimap">
            <div class="minimap-title">Tactical Mini-Map</div>
            <div class="minimap-tree">
                <!-- SVG Tree representation -->
                <svg width="100" height="80" viewBox="0 0 100 80">
                    <rect x="40" y="0" width="20" height="15" fill="#ff9900" opacity="0.8" />
                    <line x1="50" y1="15" x2="50" y2="30" stroke="white" stroke-width="1" />
                    <line x1="20" y1="30" x2="80" y2="30" stroke="white" stroke-width="1" />
                    <rect x="10" y="30" width="20" height="15" fill="#9999cc" opacity="0.6" />
                    <rect x="40" y="30" width="20" height="15" fill="#9999cc" opacity="0.4" />
                    <rect x="70" y="30" width="20" height="15" fill="#9999cc" opacity="0.4" />
                </svg>
            </div>
            <div class="minimap-stats">LEVEL 1 GLOBAL OVERVIEW<br>AUDITORY INTERFACE ACTIVE</div>
        </div>"##);

        // 3. Telemetry Bar
        html.push_str(&format!(r#"<div class="telemetry-bar">
            <div class="telemetry-item">
                <span class="label">System Time</span>
                <span class="value">{}</span>
            </div>
            <div class="telemetry-item">
                <span class="label">Ambience</span>
                <div class="ambience-controls">
                    <button class="bezel-btn" onclick="window.ipc.postMessage('play_audio:AmbientHum')">HUM</button>
                    <button class="bezel-btn" onclick="window.ipc.postMessage('play_audio:BridgeChirps')">CHIRP</button>
                </div>
            </div>
            <div class="telemetry-item">
                <span class="label">Stardate</span>
                <span class="value">{}</span>
            </div>
        </div>"#, state.get_system_time(), state.get_stardate()));

        // 4. Global Grid
        html.push_str(&format!(r#"<div class="global-grid mode-{:?}">"#, mode));
        for (i, sector) in state.sectors.iter().enumerate() {
            let color_class = match sector.color.as_str() {
                "#ff9900" => "orange",
                "#9999cc" => "blue",
                "#cc99cc" => "purple",
                _ => "green",
            };

            let desc = &sector.description;
            let icon = &sector.icon;

            let remote_indicator = match sector.connection_type {
                ConnectionType::Local => String::new(),
                ConnectionType::TOSNative => format!(r#"<div class="remote-tag">TOS // {}</div>"#, sector.host),
                ConnectionType::SSH => format!(r#"<div class="remote-tag">SSH // {}</div>"#, sector.host),
                ConnectionType::HTTP => format!(r#"<div class="remote-tag">HTTP // {}</div>"#, sector.host),
            };

            let portal_tag = if sector.portal_active {
                r#"<div class="portal-tag">PORTAL ACTIVE</div>"#
            } else {
                ""
            };

            html.push_str(&format!(
                r#"<div class="sector-card {color_class}" onclick="window.ipc.postMessage('select_sector:{index}')">
                    <div class="card-header">
                        <div class="header-label">SECTOR {index}</div>
                        {remote_indicator}
                        {portal_tag}
                        <div class="header-utils">
                            <span title="Settings">⚙️</span>
                            <span title="Pin">📌</span>
                        </div>
                    </div>
                    <div class="card-body">
                        <div class="card-icon">{icon}</div>
                        <div class="sector-name">{name}</div>
                        <div class="sector-desc">{desc}</div>
                        <div class="sector-stats">
                            <div class="stat"><span class="label">USERS</span><span class="val">{participants}</span></div>
                            <div class="stat"><span class="label">HUBS</span><span class="val">{hubs}</span></div>
                        </div>
                    </div>
                    <div class="card-footer">
                        <div class="execute-btn">ENTER</div>
                        <div class="footer-actions">
                            <button class="action-btn share-btn" onclick="event.stopPropagation(); window.ipc.postMessage('invite_participant:Viewer')">SHARE</button>
                            <button class="action-btn delete-btn" onclick="event.stopPropagation(); window.ipc.postMessage('kill_sector:{index}')">DEL</button>
                        </div>
                    </div>
                </div>"#,
                index = i,
                name = sector.name.to_uppercase(),
                icon = icon,
                desc = desc,
                participants = sector.participants.len(),
                hubs = sector.hubs.len(),
                portal_tag = portal_tag
            ));
        }
        
        // Add Remote Card (Styled like a sector card)
        html.push_str(r#"<div class="sector-card remote-card green">
            <div class="card-header">
                <div class="header-label">REMOTE CONNECTION</div>
            </div>
            <div class="card-body">
                <div class="card-icon">📡</div>
                <div class="sector-name">LINK NODE</div>
                <p class="sector-desc">Establish a persistent tactical link to a remote TOS installation via IP/Hostname.</p>
                <div class="remote-input-group">
                    <input type="text" id="remote-host-input" placeholder="ADDRESS (e.g. 192.168.1.50)" 
                           onkeyup="if(event.key==='Enter') window.ipc.postMessage('connect_remote:' + this.value)">
                </div>
            </div>
            <div class="card-footer">
                <button class="execute-btn" onclick="const val = document.getElementById('remote-host-input').value; if(val) window.ipc.postMessage('connect_remote:' + val)">CONNECT</button>
                <button class="action-btn" onclick="window.ipc.postMessage('add_remote_sector')">MOCK</button>
            </div>
        </div>"#);

        html.push_str("</div>");
        html
    }
}
