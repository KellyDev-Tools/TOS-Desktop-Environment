use crate::{TosState, Viewport, RenderMode};
use super::ViewRenderer;

pub struct DetailInspectorRenderer;

impl ViewRenderer for DetailInspectorRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];

        format!(
            r#"<div class="inspector-container detail-inspector render-{mode:?}">
                <div class="inspector-header">NODE INSPECTOR // LEVEL 4</div>
                <div class="inspector-content">
                    <div class="stat-row"><span>ID:</span> <span>{id}</span></div>
                    <div class="stat-row"><span>CLASS:</span> <span>{class}</span></div>
                    <div class="stat-row"><span>SECTOR:</span> <span>{sector}</span></div>
                    <div class="stat-row"><span>PERMISSIONS:</span> <span>0755</span></div>
                    <div class="stat-row"><span>UPTIME:</span> <span>00:14:32</span></div>
                </div>
                <div class="inspector-footer" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
            </div>"#,
            mode = mode,
            id = app.id, class = app.app_class, sector = sector.name
        )
    }
}

pub struct BufferInspectorRenderer;

impl ViewRenderer for BufferInspectorRenderer {
    fn render(&self, _state: &TosState, _viewport: &Viewport, mode: RenderMode) -> String {
        format!(r#"<div class="inspector-container buffer-inspector render-{mode:?}">
            <div class="inspector-header">BUFFER HEX DUMP // LEVEL 5</div>
            <div class="buffer-hex">
                0000: 4c 43 41 52 53 20 44 52 45 41 4d 20 43 4f 4d 50  LCARS DREAM COMP
                0010: 4c 45 54 45 20 56 45 52 53 49 4f 4e 20 31 2e 30  LETE VERSION 1.0
                0020: 0a 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f  ................
            </div>
            <div class="inspector-footer" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
        </div>"#, mode = mode)
    }
}
