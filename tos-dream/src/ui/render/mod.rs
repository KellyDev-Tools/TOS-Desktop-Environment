use crate::{TosState, Viewport, RenderMode};

pub mod global;
pub mod hub;
pub mod app;
pub mod bezel;
pub mod inspector;
pub mod remote;
pub mod confirmation;
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

pub fn render_comms_overlay(state: &TosState) -> String {
    if !state.comms_visible {
        return String::new();
    }
    
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

    format!(
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
    )
}

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#39;")
}
