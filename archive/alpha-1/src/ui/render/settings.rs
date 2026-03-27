use crate::TosState;

pub fn render_settings_modal(state: &TosState) -> String {
    if !state.settings_open {
        return String::new();
    }
    
    let fps = state.fps;

    let theme_active = "active";
    let performance_active = if state.performance_alert { "active" } else { "" };
    
    format!(
        r#"<div class="tactical-minimap active-overlay settings-modal-overlay" style="opacity: 0.98; left: 10%; right: 10%; top: 10%; bottom: 10%; z-index: 1000; display: flex; flex-direction: column;">
            <div class="minimap-header" style="flex: 0 0 auto;">
                <span class="depth-badge">CFG</span>
                <span class="header-title">SYSTEM CONFIGURATION</span>
                <div class="minimap-close" style="float: right; cursor: pointer;" onclick="window.ipc.postMessage('close_settings')">×</div>
            </div>
            
            <div class="minimap-content" style="flex: 1 1 auto; display: flex; flex-direction: row; overflow: hidden; background: rgba(0,0,0,0.8); padding: 0;">
                <div class="settings-sidebar" style="width: 250px; border-right: 2px solid var(--tactical-blue); padding: 15px; display: flex; flex-direction: column; gap: 10px;">
                    <div class="bezel-btn active" style="text-align: left; padding: 10px;">GENERAL &amp; SYSTEM</div>
                    <div class="bezel-btn" style="text-align: left; padding: 10px;">ACCESSIBILITY &amp; INPUT</div>
                    <div class="bezel-btn" style="text-align: left; padding: 10px;">TACTICAL &amp; SECURITY</div>
                    <div class="bezel-btn" style="text-align: left; padding: 10px;">SECTOR OVERRIDES</div>
                </div>
                
                <div class="settings-main-panel" style="flex: 1; padding: 20px; overflow-y: auto;">
                    <div class="bezel-section">
                        <div class="section-label">THEME &amp; APPEARANCE</div>
                        <div class="bezel-group">
                            <span style="color: var(--lcars-orange);">Data Visualization Aesthetic</span>
                            <div class="three-way-toggle" style="margin-left: 20px; flex: 1; max-width: 300px;">
                                <div class="toggle-segment {theme_active}">LCARS CURVES</div>
                                <div class="toggle-segment">HIGH-CONTRAST</div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="bezel-section" style="margin-top: 30px;">
                        <div class="section-label">PERFORMANCE &amp; THRESHOLDS</div>
                        <div class="bezel-group sliders" style="flex-direction: column; gap: 20px;">
                            <div class="action-slider">
                                <span>TARGET FPS</span>
                                <input type="range" min="30" max="144" step="1" value="{fps}" oninput="document.getElementById('fps-val-setting').innerText = this.value; window.ipc.postMessage('set_fps:' + this.value)">
                                <span id="fps-val-setting" style="min-width: 40px; color: white;">{fps}</span>
                            </div>
                            <div class="bezel-group" style="padding-top: 10px;">
                                <span style="margin-right: 20px; color: var(--lcars-orange);">Performance Mode</span>
                                <button class="bezel-btn mini {performance_active}" onclick="window.ipc.postMessage('optimize_system')">OPTIMIZE RESOURCES</button>
                            </div>
                        </div>
                    </div>
                    
                    <div class="bezel-section" style="margin-top: 30px;">
                        <div class="section-label">AUDIO &amp; HAPTICS</div>
                        <div class="bezel-group sliders">
                            <div class="action-slider">
                                <span>Master Volume</span>
                                <input type="range" min="0" max="100" step="5" value="80" onchange="window.ipc.postMessage('set_master_volume:' + this.value)">
                            </div>
                            <div class="action-slider">
                                <span>UI Feedback</span>
                                <input type="range" min="0" max="100" step="5" value="60">
                            </div>
                        </div>
                    </div>
                    
                    <div class="bezel-section" style="margin-top: 30px;">
                        <div class="section-label">SECURITY CONSTRAINTS</div>
                        <div class="bezel-group" style="gap: 15px;">
                            <button class="bezel-btn active">SANDBOXING: ENABLED</button>
                            <button class="bezel-btn active">CONFIRM DRAGS: ENABLED</button>
                            <button class="bezel-btn danger" onclick="window.ipc.postMessage('enable-deep-inspection')">ACTIVATE DEEP INSPECTION</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>"#,
        fps = fps,
        theme_active = theme_active,
        performance_active = performance_active,
    )
}
