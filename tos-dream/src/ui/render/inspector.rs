use crate::{TosState, Viewport, RenderMode};
use super::ViewRenderer;

pub struct DetailInspectorRenderer;

impl ViewRenderer for DetailInspectorRenderer {
    fn render(&self, state: &TosState, viewport: &Viewport, mode: RenderMode) -> String {
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        let app = &hub.applications[viewport.active_app_index.unwrap_or(0)];

        let mut mem_str = "--- MB".to_string();
        let mut pid_str = "N/A".to_string();
        let mut uptime_str = "00:00:00".to_string();
        let mut perm_str = "----".to_string();

        if let Some(pid) = app.pid {
            pid_str = pid.to_string();
            if let Ok(stats) = crate::system::proc::get_process_stats(pid) {
                mem_str = format!("{} MB", stats.memory_bytes / 1024 / 1024);
                
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
                <div class="inspector-header">NODE INSPECTOR // LEVEL 4</div>
                <div class="inspector-content">
                    <div class="stat-row"><span>ID:</span> <span>{id}</span></div>
                    <div class="stat-row"><span>PID:</span> <span>{pid}</span></div>
                    <div class="stat-row"><span>CLASS:</span> <span>{class}</span></div>
                    <div class="stat-row"><span>SECTOR:</span> <span>{sector}</span></div>
                    <div class="stat-row"><span>MEMORY:</span> <span>{mem}</span></div>
                    <div class="stat-row"><span>UPTIME:</span> <span>{uptime}</span></div>
                    <div class="stat-row"><span>PERMS:</span> <span>{perms}</span></div>
                </div>
                <div class="inspector-footer" onclick="window.ipc.postMessage('zoom_out')">BACK</div>
            </div>"#,
            mode = mode,
            id = app.id, 
            pid = pid_str,
            class = app.app_class, 
            sector = sector.name,
            mem = mem_str,
            uptime = uptime_str,
            perms = perm_str
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
