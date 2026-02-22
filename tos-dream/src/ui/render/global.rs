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
                r#"<div class="sector-card command-hub {color_class}" onclick="this.classList.add('expanding'); setTimeout(() => window.ipc.postMessage('select_sector:{index}'), 300)">
                    <div class="tactical-header mini" style="--header-accent: {color};">
                        <div class="header-left">
                            <button class="bezel-btn mini">ZOOM IN</button>
                            <button class="bezel-btn mini">NAV</button>
                        </div>
                        <div class="header-center">
                            <div class="three-way-toggle">
                                <div class="toggle-segment {active_cmd}">CMD</div>
                                <div class="toggle-segment {active_dir}">DIR</div>
                                <div class="toggle-segment {active_act}">ACT</div>
                            </div>
                        </div>
                        <div class="header-right">
                            <div class="hub-info-inline" style="display:flex; gap:10px; align-items:center;">
                                <div class="hub-metadata">
                                    <span class="hub-sector-name" style="font-weight:800; color:{color}; font-size: 0.8rem;">{name}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="hub-content mini" style="display:flex; flex-direction:column; justify-content:center; align-items:center; background:rgba(0,0,0,0.5);">
                        {remote_indicator}
                        {portal_tag}
                        <div style="font-size: 4rem; margin:10px 0; filter: drop-shadow(0 0 10px {color});">{icon}</div>
                        <div style="display:flex; gap:5px;">
                            <span class="pill" style="background:#4CAF50; color:black; font-size:0.6rem; padding:2px 6px; border-radius:10px;">{hubs} HUBS</span>
                            <span class="pill" style="background:#4CAF50; color:black; font-size:0.6rem; padding:2px 6px; border-radius:10px;">{participants} USERS</span>
                        </div>
                    </div>

                    <div class="unified-prompt mini">
                        <div class="center-section">
                            <div class="prompt-container" style="display:flex; align-items:center;">
                                <div class="prompt-prefix" style="color:var(--lcars-orange); font-family:var(--font-mono); font-weight:800; font-size:0.8rem; margin-right:10px;">TOS@{name} ></div>
                                <div style="width:8px; height:1rem; background:rgba(255,255,255,0.7); animation:blink-soft 1s infinite;"></div>
                            </div>
                        </div>
                    </div>
                </div>"#,
                index = i,
                name = sector.name.to_uppercase(),
                icon = icon,
                hubs = sector.hubs.len(),
                participants = sector.participants.len(),
                color_class = color_class,
                color = sector.color,
                remote_indicator = if remote_indicator.is_empty() { String::new() } else { remote_indicator.replace("class=\"remote-tag\"", "style=\"font-family:var(--font-mono);font-size:0.6rem;color:var(--lcars-orange);background:rgba(255,153,0,0.1);padding:2px 6px;border-radius:4px;font-weight:800;\"") },
                portal_tag = if portal_tag.is_empty() { String::new() } else { portal_tag.replace("class=\"portal-tag\"", "style=\"font-family:var(--font-mono);font-size:0.6rem;color:var(--lcars-purple);background:rgba(204,153,204,0.1);padding:2px 6px;border-radius:4px;font-weight:800;\"") },
                active_cmd = if sector.hubs[sector.active_hub_index].mode == crate::CommandHubMode::Command { "active" } else { "" },
                active_dir = if sector.hubs[sector.active_hub_index].mode == crate::CommandHubMode::Directory { "active" } else { "" },
                active_act = if sector.hubs[sector.active_hub_index].mode == crate::CommandHubMode::Activity { "active" } else { "" },
            ));
        }
        
        // Add Create Sector Card
        html.push_str(r#"<div class="sector-card command-hub create-card orange" onclick="window.ipc.postMessage('create_sector')">
            <div class="tactical-header mini" style="--header-accent: var(--lcars-orange);">
                <div class="header-left"><button class="bezel-btn mini">NEW</button></div>
                <div class="header-center"></div>
                <div class="header-right"><span class="hub-sector-name" style="font-weight:800; color:var(--lcars-orange); font-size:0.8rem;">INIT MODULE</span></div>
            </div>
            
            <div class="hub-content mini" style="display:flex; flex-direction:column; justify-content:center; align-items:center;">
                <div style="font-size: 4rem; opacity: 0.8; color: var(--lcars-orange);">+</div>
                <div style="font-size: 1rem; opacity: 0.6; font-weight:800; letter-spacing:2px; margin-top:10px;">CREATE SECTOR</div>
            </div>
            <div class="unified-prompt mini"></div>
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
