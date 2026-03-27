use crate::system::security::{ConfirmationSession, TactileMethod, RiskLevel};
use crate::ui::render::escape_html;

/// Render the high-fidelity tactile confirmation modal.
pub fn render_confirmation_modal(session: &ConfirmationSession, timeout_secs: u64) -> String {
    let risk_class = match session.risk_level {
        RiskLevel::Low => "risk-low",
        RiskLevel::Medium => "risk-medium",
        RiskLevel::High => "risk-high",
        RiskLevel::Critical => "risk-critical",
    };

    let risk_text = format!("{:?}", session.risk_level).to_uppercase();
    
    let interaction_html = match &session.required_method {
        TactileMethod::Slider { .. } => render_slider(session),
        TactileMethod::MultiButton { .. } => render_multibutton(session),
        TactileMethod::Hold { target, duration_ms } => render_hold(session, target, *duration_ms),
        _ => format!("<div class='fallback-interaction'>Confirmation via {:?} required. Interaction logic pending.</div>", session.required_method),
    };

    format!(
        r#"<div class="confirmation-overlay">
            <div class="confirmation-modal">
                <div class="confirmation-header">
                    <div class="header-left">
                        <div class="risk-indicator {risk_class}">{risk_text} RISK</div>
                        <div class="confirmation-title">DANGEROUS COMMAND DETECTED</div>
                    </div>
                    <div class="timer-display">{timeout}s</div>
                </div>
                
                <div class="command-container">
                    <div class="command-label">PENDING COMMAND:</div>
                    <div class="command-text"><code>{command}</code></div>
                </div>

                <div class="warning-box">
                    Tactile confirmation is required. Unconfirmed actions will be automatically aborted when the timer expires.
                </div>

                <div class="tactile-interaction-area">
                    {interaction_html}
                </div>

                <div class="modal-actions">
                    <button class="abort-btn" onclick="window.ipc.postMessage('cancel_confirmation:{id}')">ABORT ACTION</button>
                </div>
            </div>
        </div>"#,
        risk_class = risk_class,
        risk_text = risk_text,
        timeout = timeout_secs,
        command = escape_html(&session.command),
        interaction_html = interaction_html,
        id = session.id
    )
}

fn render_slider(session: &ConfirmationSession) -> String {
    let progress_pct = (session.progress * 100.0).clamp(0.0, 100.0);
    
    format!(
        r#"<div class="slider-container">
            <div class="slider-track">
                <div class="slider-progress" style="width: {progress}%"></div>
                <input type="range" class="confirm-slider-input" min="0" max="100" value="{progress}" 
                    style="position:absolute; top:0; left:0; width:100%; height:100%; opacity:0; cursor:pointer;"
                    oninput="window.ipc.postMessage('update_confirmation_progress:{id}:' + (this.value/100))"
                    onchange="if(this.value < 100) {{ this.value = 0; window.ipc.postMessage('update_confirmation_progress:{id}:0') }} else {{ window.ipc.postMessage('confirm_command:{id}') }}">
                <div class="slider-thumb" style="left: calc({progress}% - {offset}px)">
                    {thumb_text}
                </div>
                <div class="slider-label">SLIDE TO CONFIRM</div>
            </div>
        </div>"#,
        id = session.id,
        progress = progress_pct,
        offset = (progress_pct / 100.0) * 100.0,
        thumb_text = if progress_pct >= 95.0 { "RELEASE" } else { "SLIDE" }
    )
}

fn render_multibutton(session: &ConfirmationSession) -> String {
    if let TactileMethod::MultiButton { buttons, .. } = &session.required_method {
        let mut indexed_buttons: Vec<(usize, &String)> = buttons.iter().enumerate().collect();
        
        // Use a simple deterministic shuffle based on session ID for "randomized" but stable UI
        let id_bytes = session.id.as_bytes();
        let seed = id_bytes[0] as usize;
        for i in (1..indexed_buttons.len()).rev() {
            let j = (seed + i) % (i + 1);
            indexed_buttons.swap(i, j);
        }

        let mut buttons_html = String::new();
        let pressed_count = (session.progress).floor() as usize;
        
        for (i, btn) in indexed_buttons {
            let active_class = if i < pressed_count { "active" } else { "" };
            let danger_class = if btn.to_lowercase() == "delete" || btn.to_lowercase() == "execute" || btn.to_lowercase() == "ctrl" { "danger" } else { "" };
            
            buttons_html.push_str(&format!(
                r#"<div class="confirm-btn {active_class} {danger_class}" onclick="window.ipc.postMessage('update_confirmation_progress:{id}:{next_progress}')">
                    <span class="btn-label">{}</span>
                    <span class="btn-key">STEP {}</span>
                </div>"#,
                btn.to_uppercase(),
                i + 1,
                id = session.id,
                next_progress = i + 1
            ));
        }

        format!(
            r#"<div class="multibutton-grid">
                {}
            </div>"#,
            buttons_html
        )
    } else {
        String::new()
    }
}

fn render_hold(session: &ConfirmationSession, target: &str, duration_ms: u64) -> String {
    let progress_pct = (session.progress * 100.0).clamp(0.0, 100.0);
    let charging_glow = if session.progress > 0.1 { "charging-glow" } else { "" };
    
    format!(
        r#"<div class="slider-container hold-container">
            <div class="slider-track hold-track {charging_glow}">
                <div class="slider-progress charging-bar" style="width: {progress}%"></div>
                <button class="confirm-btn active" style="width:100%; height:100%; border-radius:30px; position:relative; z-index:2; background:transparent;"
                    onmousedown="window.holdInterval = setInterval(() => window.ipc.postMessage('increment_hold:{id}'), 100)"
                    onmouseup="clearInterval(window.holdInterval); window.ipc.postMessage('reset_hold:{id}')"
                    onmouseleave="clearInterval(window.holdInterval); window.ipc.postMessage('reset_hold:{id}')">
                    HOLD {target} ({duration}s)
                </button>
            </div>
            <div class="slider-label" style="margin-top:10px; position:static; transform:none;">TACTILE CHARGE: {progress:.0}%</div>
        </div>"#,
        id = session.id,
        target = target.to_uppercase(),
        duration = duration_ms / 1000,
        progress = progress_pct,
        charging_glow = charging_glow
    )
}
