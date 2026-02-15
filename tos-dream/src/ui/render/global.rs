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
        html.push_str(r#"<div class="telemetry-bar">
            <div class="telemetry-item">
                <span class="label">System Time</span>
                <span class="value">10:39</span>
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
                <span class="value">02-33 // 02-1478</span>
            </div>
        </div>"#);

        // 4. Global Grid
        html.push_str(&format!(r#"<div class="global-grid mode-{:?}">"#, mode));
        for (i, sector) in state.sectors.iter().enumerate() {
            let color_class = match sector.color.as_str() {
                "#ff9900" => "orange",
                "#9999cc" => "blue",
                "#cc99cc" => "purple",
                _ => "green",
            };

            let (desc, icon) = match sector.name.as_str() {
                "Alpha Sector" => ("Primary coordination and terminal access.", "⌨️"),
                "Science Labs" => ("Data analysis and sensor array telemetry.", "🔬"),
                "Engineering" => ("Core systems and resource management.", "⚙️"),
                _ => ("Remote node established via TOS protocol.", "📡"),
            };

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
                            <span>+</span>
                            <span>✎</span>
                        </div>
                    </div>
                    <div class="card-body">
                        <div class="card-icon" style="font-size: 3rem; margin-top: 10px;">{icon}</div>
                        <div class="sector-name">{name}</div>
                        <div class="sector-desc">{desc}</div>
                    </div>
                    <div class="card-footer">
                        <div class="execute-btn">Execute</div>
                        <div class="header-utils"><span style="opacity: 0.5">🗑</span></div>
                    </div>
                </div>"#,
                index = i,
                name = sector.name.to_uppercase(),
                icon = icon,
                desc = desc,
                portal_tag = portal_tag
            ));
        }
        
        // Add Remote Card (Styled like a sector card)
        html.push_str(r#"<div class="sector-card green" onclick="window.ipc.postMessage('add_remote_sector')">
            <div class="card-header">
                <div class="header-label">REMOTE</div>
            </div>
            <div class="card-body">
                <div class="card-icon" style="font-size: 3rem; margin-top: 10px;">📡</div>
                <div class="sector-name">ADD REMOTE</div>
                <div class="sector-desc">Establish link to remote TOS node.</div>
            </div>
            <div class="card-footer">
                <div class="execute-btn">Link</div>
            </div>
        </div>"#);

        html.push_str("</div>");
        html
    }
}
