use crate::{TosState, Viewport, RenderMode, ConnectionType};
use super::ViewRenderer;

pub struct GlobalRenderer;

impl ViewRenderer for GlobalRenderer {
    fn render(&self, state: &TosState, _viewport: &Viewport, mode: RenderMode) -> String {
        let mut html = String::new();

        // 1. Unified Tactical Header
        html.push_str(&format!(
            r#"<div class="tactical-header" style="--header-accent: var(--lcars-blue);">
                <div class="header-left">
                    <h1 class="header-title large">TOS COMMAND CENTER</h1>
                </div>
                <div class="header-center"></div>
                <div class="header-right">
                    <div class="telemetry-bar-inline" style="display:flex; gap:30px; align-items:center;">
                        <div class="telemetry-item">
                            <span class="label">System Time</span>
                            <span class="value" id="tos-sys-time" style="color:var(--lcars-orange); font-weight:700; font-size:1.2rem;">{}</span>
                        </div>
                        <div class="telemetry-item">
                            <span class="label">Ambience</span>
                            <div class="ambience-controls" style="display:flex; gap:5px;">
                                <button class="bezel-btn mini" onclick="window.ipc.postMessage('play_audio:AmbientHum')">HUM</button>
                                <button class="bezel-btn mini" onclick="window.ipc.postMessage('play_audio:BridgeChirps')">CHIRP</button>
                            </div>
                        </div>
                        <div class="telemetry-item" style="display:flex; gap:5px;">
                            <button class="bezel-btn mini comms-toggle-btn" onclick="window.ipc.postMessage('toggle_comms')">COMMS</button>
                            <button class="bezel-btn mini minimap-toggle-btn" onclick="window.ipc.postMessage('semantic_event:ToggleMiniMap')">MAP</button>
                        </div>
                        <div class="telemetry-item">
                            <span class="label">Stardate</span>
                            <span class="value" id="tos-stardate" style="color:var(--lcars-orange); font-weight:700; font-size:1.2rem;">{}</span>
                        </div>
                    </div>
                </div>
            </div>"#,
            state.get_system_time(),
            state.get_stardate()
        ));

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
                        {remote_indicator}
                        {portal_tag}
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
                remote_indicator = remote_indicator,
                portal_tag = portal_tag,
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
        html.push_str(r#"<div class="sector-card create-card orange" onclick="window.ipc.postMessage('create_sector')">
            <div class="mini-bezel">NEW MODULE</div>
            <div class="left-indicators"></div>
            <div class="card-body" style="align-items: center; justify-content: center;">
                <div class="card-icon" style="margin-bottom: 10px; font-size: 4rem;">+</div>
                <div class="sector-name" style="font-size: 1.2rem; opacity: 0.6;">CREATE SECTOR</div>
            </div>
            <div class="right-indicators"></div>
            <div class="mini-prompt"></div>
        </div>"#);

        // Add Remote Card (Styled like a sector card)
        html.push_str(r#"<div class="sector-card remote-card green">
            <div class="mini-bezel">REMOTE CONNECTION</div>
            <div class="left-indicators"></div>
            <div class="card-body">
                <div class="card-icon">📡</div>
                <div class="sector-name" style="font-size: 1.5rem;">LINK NODE</div>
                <p class="sector-desc">Establish a persistent tactical link to a remote TOS installation via IP/Hostname.</p>
                <div class="remote-input-group">
                    <input type="text" id="remote-host-input" placeholder="ADDRESS (e.g. 192.168.1.50)" 
                           onkeyup="if(event.key==='Enter') window.ipc.postMessage('connect_remote:' + this.value)">
                </div>
                <div style="margin-top: 15px;">
                    <button class="execute-btn" onclick="const val = document.getElementById('remote-host-input').value; if(val) window.ipc.postMessage('connect_remote:' + val)">CONNECT</button>
                </div>
            </div>
            <div class="right-indicators"></div>
            <div class="mini-prompt"></div>
        </div>"#);

        html.push_str("</div>");

        html
    }
}
