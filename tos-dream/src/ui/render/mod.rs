use crate::{TosState, Viewport, RenderMode};

pub mod global;
pub mod hub;
pub mod app;
pub mod inspector;
pub mod remote;
pub mod svg_engine;

pub trait ViewRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String;
}

pub fn render_performance_overlay(fps: f32, alert: bool) -> String {
    if !alert {
        return String::new();
    }
    format!(
        r#"<div class="tactical-alert perflo-alert">
            <div class="alert-title">TACTICAL ALERT // PERFORMANCE CRITICAL</div>
            <div class="alert-stats">CURRENT FPS: {fps:.1} // DEPTH THROTTLE ACTIVE</div>
            <div class="alert-actions">
                <button onclick="window.ipc.postMessage('optimize_system')">OPTIMIZE RESOURCES</button>
                <button onclick="window.ipc.postMessage('tactical_reset')">TACTICAL RESET</button>
            </div>
        </div>"#,
        fps = fps
    )
}
