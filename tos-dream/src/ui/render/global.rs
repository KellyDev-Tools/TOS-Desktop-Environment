use crate::{TosState, Viewport, RenderMode, ConnectionType};
use super::ViewRenderer;

pub struct GlobalRenderer;

impl ViewRenderer for GlobalRenderer {
    fn render(&self, state: &TosState, _viewport: &Viewport, mode: RenderMode) -> String {
        let mut html = String::new();

        // 4. Global Grid
        html.push_str(&format!(r#"<div class="global-grid mode-{:?}">"#, mode));

        // System time and stardate elements (required by tests and UI)
        html.push_str(r#"<div class="global-header" style="display:flex; justify-content:flex-end; gap:12px; padding:8px 12px;">
            <div id="tos-stardate" style="font-family:var(--font-mono); font-weight:700; color:rgba(255,255,255,0.85);">STARDATE 0.0</div>
            <div id="tos-sys-time" style="font-family:var(--font-mono); font-weight:700; color:rgba(255,255,255,0.85);">00:00</div>
        </div>"#);
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
                let url = sector.portal_url.as_deref().unwrap_or("TOS://PORTAL");
                format!(
                    r#"<div class="portal-tag" onclick="event.stopPropagation(); window.open('{url}', '_blank')">
                        <div class="portal-status">PORTAL ACTIVE</div>
                        <div class="portal-url">{url}</div>
                    </div>"#
                )
            } else {
                String::new()
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
                        <div style="font-size: 0.8rem; text-align: center; margin-bottom: 10px; color: rgba(255,255,255,0.8);">{desc}</div>
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
                desc = sector.description,
                hubs = sector.hubs.len(),
                participants = sector.participants.len(),
                color_class = color_class,
                color = sector.color,
                remote_indicator = remote_indicator,
                portal_tag = portal_tag,
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
