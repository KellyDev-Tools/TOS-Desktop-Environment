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
                                <input type="range" min="1" max="10" value="5">
                             </div>
                             <div class="action-slider">
                                <span>POWER</span>
                                <input type="range" min="1" max="100" value="80">
                             </div>
                        </div>
                    </div>
                </div>
                <div class="application-surface" onclick="window.ipc.postMessage('zoom_in')">
                    <div class="app-mock-content">
                        APPLICATION DATA FEED: {title}
                        {module_content}
                    </div>
                </div>
            </div>"#,
            mode = mode,
            bezel_class = bezel_class,
            title = app.title.to_uppercase(),
            class = app.app_class.to_uppercase(),
            participants_html = participants_html,
            module_content = module_content,
            portal_active_class = portal_active_class,
            portal_label = portal_label,
            portal_info_html = portal_info_html
        )
    }
}
