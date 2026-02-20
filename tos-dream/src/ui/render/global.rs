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
                <span class="value" id="tos-sys-time">{}</span>
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
                <span class="value" id="tos-stardate">{}</span>
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
                    <div class="mini-bezel">
                        {remote_label}
                        <div class="settings-trigger" onclick="event.stopPropagation(); window.ipc.postMessage('open_settings')">⚙️</div>
                    </div>
                    <div class="left-indicators">
                        <div class="mode-chip {active_cmd}"></div>
                        <div class="mode-chip {active_dir}"></div>
                        <div class="mode-chip {active_act}"></div>
                    </div>
                    <div class="card-body">
                        <div class="collaboration-dots">
                            {collab_dots}
                        </div>
                        <div class="card-icon">{icon}</div>
                        <div class="sector-name">{name}</div>
                        <div class="sector-desc">{desc}</div>
                        <div class="sector-stats">
                            <div class="stat"><span class="val">{hubs} HUBS</span></div>
                            <div class="stat"><span class="val">{participants} USERS</span></div>
                        </div>
                    </div>
                    <div class="right-indicators">
                        {priority_chips}
                    </div>
                    <div class="mini-prompt"></div>
                </div>"#,
                index = i,
                name = sector.name.to_uppercase(),
                icon = icon,
                desc = desc,
                hubs = sector.hubs.len(),
                participants = sector.participants.len(),
                color_class = color_class,
                remote_label = match sector.connection_type {
                    ConnectionType::Local => "LOCAL".to_string(),
                    ConnectionType::TOSNative => format!("TOS // {}", sector.host),
                    ConnectionType::SSH => format!("SSH // {}", sector.host),
                    ConnectionType::HTTP => format!("HTTP // {}", sector.host),
                },
                active_cmd = if sector.hubs[sector.active_hub_index].mode == crate::CommandHubMode::Command { "active" } else { "" },
                active_dir = if sector.hubs[sector.active_hub_index].mode == crate::CommandHubMode::Directory { "active" } else { "" },
                active_act = if sector.hubs[sector.active_hub_index].mode == crate::CommandHubMode::Activity { "active" } else { "" },
                collab_dots = sector.participants.iter().map(|_| r#"<div class="collab-dot"></div>"#).collect::<Vec<_>>().join(""),
                priority_chips = {
                    let score = sector.priority_score(state);
                    let active_chips = (score / 3.0).min(5.0) as usize;
                    let mut chips = String::new();
                    for j in 0..5 {
                        let active = if j < active_chips { "active" } else { "" };
                        chips.push_str(&format!(r#"<div class="priority-chip-mini {}"></div>"#, active));
                    }
                    chips
                }
            ));
        }
        
        // Add Create Sector Card
        html.push_str(r#"<div class="sector-card create-card" onclick="window.ipc.postMessage('create_sector')">
            <div class="card-body" style="align-items: center; justify-content: center;">
                <div class="card-icon" style="margin-bottom: 10px; font-size: 4rem;">+</div>
                <div class="sector-name" style="font-size: 1.2rem; opacity: 0.6;">NEW SECTOR</div>
            </div>
        </div>"#);

        // Add Remote Card (Styled like a sector card)
        html.push_str(r#"<div class="sector-card remote-card green">
            <div class="card-header">
                <div class="header-label">REMOTE CONNECTION</div>
            </div>
            <div class="card-body">
                <div class="card-icon">📡</div>
                <div class="sector-name" style="font-size: 1.5rem;">LINK NODE</div>
                <p class="sector-desc">Establish a persistent tactical link to a remote TOS installation via IP/Hostname.</p>
                <div class="remote-input-group">
                    <input type="text" id="remote-host-input" placeholder="ADDRESS (e.g. 192.168.1.50)" 
                           onkeyup="if(event.key==='Enter') window.ipc.postMessage('connect_remote:' + this.value)">
                </div>
            </div>
            <div class="card-footer">
                <button class="execute-btn" onclick="const val = document.getElementById('remote-host-input').value; if(val) window.ipc.postMessage('connect_remote:' + val)">CONNECT</button>
            </div>
        </div>"#);

        html.push_str("</div>");

        // 5. Comms Overlay (Advanced Services)
        let mut comms_html = String::new();
        for msg in state.comms_messages.iter().rev().take(10).rev() {
            comms_html.push_str(&format!(
                r#"<div class="comms-msg">
                    <span class="comms-time">[{}]</span>
                    <span class="comms-from">{}:</span>
                    <span class="comms-body">{}</span>
                </div>"#,
                msg.timestamp, msg.from, msg.body
            ));
        }

        html.push_str(&format!(
            r#"<div class="comms-overlay">
                <div class="comms-header">DIRECT COMMS // ENCRYPTED</div>
                <div class="comms-list">
                    {}
                </div>
                <div class="comms-input-area">
                    <input type="text" class="comms-input" placeholder="BROADCAST..." 
                           onkeyup="if(event.key==='Enter') {{ window.ipc.postMessage('send_comms:' + this.value); this.value = ''; }}">
                </div>
            </div>"#,
            comms_html
        ));

        html
    }
}
