use crate::{TosState, Viewport, RenderMode};
use super::ViewRenderer;

pub struct DetailInspectorRenderer;

impl ViewRenderer for DetailInspectorRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];

        let mut mem_str = "--- MB".to_string();
        let mut cpu_str = "0.0%".to_string();
        let mut pid_str = "N/A".to_string();
        let mut uptime_str = "00:00:00".to_string();
        let mut perm_str = "----".to_string();

        if let Some(pid) = app.pid {
            pid_str = pid.to_string();
            if let Ok(stats) = crate::system::proc::get_process_stats(pid) {
                mem_str = format!("{} MB", stats.memory_bytes / 1024 / 1024);
                cpu_str = format!("{:.1}%", stats.cpu_usage);
                
                let total_seconds = stats.uptime_seconds as u64;
                let hours = total_seconds / 3600;
                let mins = (total_seconds % 3600) / 60;
                let secs = total_seconds % 60;
                uptime_str = format!("{:02}:{:02}:{:02}", hours, mins, secs);
                
                perm_str = format!("UID: {:04}", stats.uid);
            }
        }

        format!(
            r#"<div class="inspector-container detail-inspector render-{mode:?}">
                <div class="inspector-header">
                    <div class="header-main">NODE INSPECTOR // LEVEL 4</div>
                    <div class="header-sub">{title} // {id}</div>
                </div>
                
                <div class="inspector-grid">
                    <div class="inspector-section">
                        <div class="section-title">SYSTEM RESOURCES</div>
                        <div class="section-body">
                            <div class="stat-row"><span class="label">PID</span> <span class="val">{pid}</span></div>
                            <div class="stat-row"><span class="label">CPU USAGE</span> <span class="val">{cpu}</span></div>
                            <div class="stat-row"><span class="label">MEM RESIDENT</span> <span class="val">{mem}</span></div>
                            <div class="stat-row"><span class="label">UPTIME</span> <span class="val">{uptime}</span></div>
                        </div>
                    </div>

                    <div class="inspector-section">
                        <div class="section-title">SECURITY & SCOPE</div>
                        <div class="section-body">
                            <div class="stat-row"><span class="label">PERMISSIONS</span> <span class="val">{perms}</span></div>
                            <div class="stat-row"><span class="label">SANDBOX</span> <span class="val">ACTIVE</span></div>
                            <div class="stat-row"><span class="label">DECORATION</span> <span class="val">{deco:?}</span></div>
                            <div class="bezel-btn mini" onclick="window.ipc.postMessage('zoom_to:BufferInspector')">MEMORY DUMP</div>
                        </div>
                    </div>

                    <div class="inspector-section full-width">
                        <div class="section-title">COLLABORATION GRAPH</div>
                        <div class="section-body collab-list">
                            <div class="stat-row"><span class="label">PARTICIPANTS</span> <span class="val">{participants}</span></div>
                            <div class="stat-row"><span class="label">FOLLOW MODE</span> <span class="val">OFF</span></div>
                        </div>
                    </div>
                </div>

                <div class="inspector-footer">
                    <button class="bezel-btn" onclick="window.ipc.postMessage('zoom_out')">RETURN TO OVERVIEW</button>
                    <button class="bezel-btn danger" onclick="window.ipc.postMessage('kill_app')">TERMINATE NODE</button>
                </div>
            </div>"#,
            mode = mode,
            title = app.title.to_uppercase(),
            id = &app.id.to_string()[..8], 
            pid = pid_str,
            cpu = cpu_str,
            mem = mem_str,
            uptime = uptime_str,
            perms = perm_str,
            deco = app.decoration_policy,
            participants = sector.participants.len()
        )
    }
}

pub struct BufferInspectorRenderer;

impl ViewRenderer for BufferInspectorRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];
        
        // Security Check: Level 5 requires Deep Inspection privilege (§11.6)
        if !state.security.deep_inspection_active {
            return format!(r#"<div class="inspector-container buffer-inspector render-{mode:?} access-denied">
                <div class="inspector-header">BUFFER HEX DUMP // LEVEL 5</div>
                <div class="access-denied-message">
                    <div class="lock-icon">🔒</div>
                    <div class="denied-title">ACCESS DENIED</div>
                    <div class="denied-reason">DEEP INSPECTION PRIVILEGE REQUIRED</div>
                    <div class="denied-instruction">Execute 'enable-deep-inspection' to unlock</div>
                </div>
                <div class="inspector-footer" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
            </div>"#, mode = mode);
        }

        // Get buffer data (cmdline + environ)
        let buffer = if let Some(pid) = app.pid {
            crate::system::proc::get_process_buffer_sample(pid)
        } else {
            // Mock buffer for dummy apps
            let mut b = Vec::new();
            b.extend_from_slice(format!("MOCK BUFFER FOR {}", app.title).as_bytes());
            b.resize(512, 0);
            b
        };
        
        let mut hex_html = String::new();
        // Show first 256 bytes (16 lines)
        for (i, chunk) in buffer.iter().take(256).cloned().collect::<Vec<u8>>().chunks(16).enumerate() {
            let offset = i * 16;
            let hex_bytes: String = chunk.iter().map(|b| format!("{:02x} ", b)).collect();
            let ascii: String = chunk.iter().map(|b| if *b >= 32 && *b <= 126 { *b as char } else { '.' }).collect();
            
            hex_html.push_str(&format!(
                "{:04x}: {:48}  {}\n", 
                offset, hex_bytes, ascii
            ));
        }
        
        format!(r#"<div class="inspector-container buffer-inspector render-{mode:?}">
            <div class="inspector-header">BUFFER HEX DUMP // LEVEL 5</div>
            <div class="buffer-hex">
{hex_dump}
            </div>
            <div class="inspector-footer" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
        </div>"#, mode = mode, hex_dump = hex_html)
    }
}
