// Development Monitor: HTTP + WebSocket server for browser-based UI debugging
// Surfaces the full TOS infrastructure: viewports, compositor, GPU pipeline, PTY
//
// This allows you to watch the compositor in real-time via a browser,
// including multi-viewport state, Wayland surface lifecycle, GPU cache stats,
// and shell PTY session output.

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::sync::broadcast;

pub type ClientId = usize;

/// Message to broadcast to all connected browser clients
#[derive(Debug, Clone)]
pub enum BroadcastMessage {
    // ─── Original Messages ───────────────────────────
    UpdateViewport {
        html_content: String,
        zoom_level: u8,
        is_red_alert: bool,
    },
    UpdateDashboard(String),
    ZoomLevel(u8),
    TestEvent {
        test_name: String,
        event_type: String, // "started", "assertion", "completed", "step"
        details: String,
    },

    // ─── Multi-Viewport State ────────────────────────
    ViewportState {
        viewports: Vec<ViewportInfo>,
        focused_id: Option<u32>,
        output_count: usize,
    },

    // ─── Wayland Compositor State ────────────────────
    CompositorState {
        surfaces: Vec<SurfaceInfo>,
        client_count: usize,
        focused_surface: Option<u64>,
        events: Vec<String>, // Recent event descriptions
    },

    // ─── GPU Pipeline Stats ──────────────────────────
    GpuStats {
        frame_count: u64,
        texture_count: usize,
        vram_used_mb: f64,
        vram_budget_mb: f64,
        vram_usage_pct: f64,
        dirty_textures: usize,
        active_transition: Option<TransitionInfo>,
        fps: u32,
    },

    // ─── PTY Session Updates ─────────────────────────
    PtyOutput {
        surface_id: u32,
        text: String,
    },
    PtyEvent {
        surface_id: u32,
        event_type: String,
        details: String,
    },
    PtySessionsState {
        sessions: Vec<PtySessionInfo>,
    },

    // ─── System-Wide Snapshot ─────────────────────────
    /// A full state snapshot combining all subsystems (sent on client connect)
    FullSnapshot {
        viewports: Vec<ViewportInfo>,
        focused_viewport: Option<u32>,
        surfaces: Vec<SurfaceInfo>,
        compositor_clients: usize,
        gpu_frame_count: u64,
        gpu_vram_used_mb: f64,
        gpu_vram_budget_mb: f64,
        pty_sessions: Vec<PtySessionInfo>,
        zoom_level: u8,
        is_red_alert: bool,
    },

    // ─── Browser Viewport Feedback ───────────────────
    /// Reported by the connected browser so the backend can scale content
    ClientViewport {
        /// CSS pixel width of the main content area
        width: u32,
        /// CSS pixel height of the main content area
        height: u32,
        /// Device pixel ratio (e.g. 2.0 for Retina)
        device_pixel_ratio: f64,
        /// Whether the dev panel sidebar is open (reduces usable width)
        dev_panel_open: bool,
    },
}

/// Viewport info for the dev monitor
#[derive(Debug, Clone)]
pub struct ViewportInfo {
    pub id: u32,
    pub label: String,
    pub output_name: String,
    pub zoom_path: String,     // Display string like "[0 → 2 → 7]"
    pub zoom_depth: usize,
    pub current_level: String, // "Level1Root", "Level2Sector", etc.
    pub has_focus: bool,
    pub geometry: [f64; 4],    // [x, y, w, h]
}

/// Surface info for the dev monitor
#[derive(Debug, Clone)]
pub struct SurfaceInfo {
    pub id: u64,
    pub client_id: u32,
    pub role: String,      // "Toplevel", "XWayland", "Popup", etc.
    pub title: String,
    pub app_id: String,
    pub size: [u32; 2],
    pub position: [i32; 2],
    pub sector: Option<u32>,
    pub activated: bool,
    pub committed: bool,
}

/// GPU transition info
#[derive(Debug, Clone)]
pub struct TransitionInfo {
    pub direction: String, // "zoom-in" or "zoom-out"
    pub progress: f32,
    pub duration_ms: u32,
}

/// PTY session info
#[derive(Debug, Clone)]
pub struct PtySessionInfo {
    pub surface_id: u32,
    pub shell_path: String,
    pub shell_name: String,
    pub cwd: String,
}

/// Development Monitor Server
pub struct DevMonitor {
    port: u16,
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
    clients: Arc<Mutex<HashMap<ClientId, String>>>,
}

impl DevMonitor {
    /// Create a new dev monitor on the specified port
    pub fn new(port: u16) -> Self {
        let (broadcast_tx, _) = broadcast::channel(256); // Larger buffer for new message types
        Self {
            port,
            broadcast_tx,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a sender handle for broadcasting messages
    pub fn get_broadcaster(&self) -> DevMonitorBroadcaster {
        DevMonitorBroadcaster {
            tx: self.broadcast_tx.clone(),
        }
    }

    /// Start the HTTP and WebSocket server (async)
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        use warp::Filter;
        use futures_util::{StreamExt, SinkExt};

        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();

        println!("[Dev Monitor] Starting on http://{}", addr);
        println!("[Dev Monitor] WebSocket on ws://{}/ws", addr);

        // Serve static files from ui/ directory at root
        let assets = warp::path("assets")
            .and(warp::fs::dir("./ui/assets"));

        // Serve index.html at root
        let index = warp::path::end()
            .and(warp::fs::file("./ui/index.html"));

        let clients = self.clients.clone();
        let broadcast_tx = self.broadcast_tx.clone();
        let next_client_id = Arc::new(Mutex::new(0usize));

        // WebSocket endpoint
        let ws_route = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let clients = clients.clone();
                let broadcast_tx = broadcast_tx.clone();
                let next_id = Arc::clone(&next_client_id);

                ws.on_upgrade(move |websocket| async move {
                    let client_id = {
                        let mut id = next_id.lock().unwrap();
                        let current = *id;
                        *id += 1;
                        current
                    };

                    println!("[Dev Monitor] Client {} connected", client_id);
                    clients.lock().unwrap().insert(client_id, format!("Browser {}", client_id));

                    let (mut ws_tx, mut ws_rx) = websocket.split();
                    let mut rx = broadcast_tx.subscribe();

                    // Spawn task to forward broadcast messages to this client
                    let forward_task = tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await {
                            let json = serialize_message(&msg);
                            let text = serde_json::to_string(&json).unwrap();
                            if ws_tx.send(warp::ws::Message::text(text)).await.is_err() {
                                break;
                            }
                        }
                    });

                    // Handle incoming messages from browser
                    let broadcast_tx_in = broadcast_tx.clone();
                    while let Some(result) = ws_rx.next().await {
                        if let Ok(msg) = result {
                            if let Ok(text) = msg.to_str() {
                                // Parse JSON commands from the browser
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                                    match json.get("type").and_then(|t| t.as_str()) {
                                        Some("viewport_size") => {
                                            let w = json.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                            let h = json.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                            let dpr = json.get("devicePixelRatio").and_then(|v| v.as_f64()).unwrap_or(1.0);
                                            let panel = json.get("devPanelOpen").and_then(|v| v.as_bool()).unwrap_or(false);
                                            println!("[Dev Monitor] Client {} viewport: {}x{} @{:.1}x (panel: {})",
                                                client_id, w, h, dpr, if panel { "open" } else { "closed" });
                                            let _ = broadcast_tx_in.send(BroadcastMessage::ClientViewport {
                                                width: w,
                                                height: h,
                                                device_pixel_ratio: dpr,
                                                dev_panel_open: panel,
                                            });
                                        }
                                        Some("command") => {
                                            let cmd = json.get("data").and_then(|v| v.as_str()).unwrap_or("");
                                            println!("[Dev Monitor] Client {} command: {}", client_id, cmd);
                                            // TODO: Forward to DesktopEnvironment
                                        }
                                        _ => {
                                            println!("[Dev Monitor] Client {} → {}", client_id, text);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    println!("[Dev Monitor] Client {} disconnected", client_id);
                    clients.lock().unwrap().remove(&client_id);
                    forward_task.abort();
                })
            });

        let routes = index.or(assets).or(ws_route);

        warp::serve(routes).run(addr).await;

        Ok(())
    }
}

/// Serialize a BroadcastMessage to a serde_json::Value
fn serialize_message(msg: &BroadcastMessage) -> serde_json::Value {
    match msg {
        BroadcastMessage::UpdateViewport { html_content, zoom_level, is_red_alert } => {
            serde_json::json!({
                "type": "viewport",
                "html": html_content,
                "zoom": zoom_level,
                "redAlert": is_red_alert,
            })
        }
        BroadcastMessage::UpdateDashboard(html) => {
            serde_json::json!({
                "type": "dashboard",
                "html": html,
            })
        }
        BroadcastMessage::ZoomLevel(level) => {
            serde_json::json!({
                "type": "zoom",
                "level": level,
            })
        }
        BroadcastMessage::TestEvent { test_name, event_type, details } => {
            serde_json::json!({
                "type": "test_event",
                "test": test_name,
                "event": event_type,
                "details": details,
            })
        }
        BroadcastMessage::ViewportState { viewports, focused_id, output_count } => {
            serde_json::json!({
                "type": "viewport_state",
                "viewports": viewports.iter().map(|v| serde_json::json!({
                    "id": v.id,
                    "label": v.label,
                    "outputName": v.output_name,
                    "zoomPath": v.zoom_path,
                    "zoomDepth": v.zoom_depth,
                    "currentLevel": v.current_level,
                    "hasFocus": v.has_focus,
                    "geometry": v.geometry,
                })).collect::<Vec<_>>(),
                "focusedId": focused_id,
                "outputCount": output_count,
            })
        }
        BroadcastMessage::CompositorState { surfaces, client_count, focused_surface, events } => {
            serde_json::json!({
                "type": "compositor_state",
                "surfaces": surfaces.iter().map(|s| serde_json::json!({
                    "id": s.id,
                    "clientId": s.client_id,
                    "role": s.role,
                    "title": s.title,
                    "appId": s.app_id,
                    "size": s.size,
                    "position": s.position,
                    "sector": s.sector,
                    "activated": s.activated,
                    "committed": s.committed,
                })).collect::<Vec<_>>(),
                "clientCount": client_count,
                "focusedSurface": focused_surface,
                "events": events,
            })
        }
        BroadcastMessage::GpuStats {
            frame_count, texture_count, vram_used_mb, vram_budget_mb,
            vram_usage_pct, dirty_textures, active_transition, fps
        } => {
            let transition_json = active_transition.as_ref().map(|t| serde_json::json!({
                "direction": t.direction,
                "progress": t.progress,
                "durationMs": t.duration_ms,
            }));
            serde_json::json!({
                "type": "gpu_stats",
                "frameCount": frame_count,
                "textureCount": texture_count,
                "vramUsedMb": vram_used_mb,
                "vramBudgetMb": vram_budget_mb,
                "vramUsagePct": vram_usage_pct,
                "dirtyTextures": dirty_textures,
                "activeTransition": transition_json,
                "fps": fps,
            })
        }
        BroadcastMessage::PtyOutput { surface_id, text } => {
            serde_json::json!({
                "type": "pty_output",
                "surfaceId": surface_id,
                "text": text,
            })
        }
        BroadcastMessage::PtyEvent { surface_id, event_type, details } => {
            serde_json::json!({
                "type": "pty_event",
                "surfaceId": surface_id,
                "event": event_type,
                "details": details,
            })
        }
        BroadcastMessage::PtySessionsState { sessions } => {
            serde_json::json!({
                "type": "pty_sessions",
                "sessions": sessions.iter().map(|s| serde_json::json!({
                    "surfaceId": s.surface_id,
                    "shellPath": s.shell_path,
                    "shellName": s.shell_name,
                    "cwd": s.cwd,
                })).collect::<Vec<_>>(),
            })
        }
        BroadcastMessage::FullSnapshot {
            viewports, focused_viewport, surfaces, compositor_clients,
            gpu_frame_count, gpu_vram_used_mb, gpu_vram_budget_mb,
            pty_sessions, zoom_level, is_red_alert,
        } => {
            serde_json::json!({
                "type": "full_snapshot",
                "viewports": viewports.iter().map(|v| serde_json::json!({
                    "id": v.id,
                    "label": v.label,
                    "outputName": v.output_name,
                    "zoomPath": v.zoom_path,
                    "zoomDepth": v.zoom_depth,
                    "currentLevel": v.current_level,
                    "hasFocus": v.has_focus,
                    "geometry": v.geometry,
                })).collect::<Vec<_>>(),
                "focusedViewport": focused_viewport,
                "surfaces": surfaces.iter().map(|s| serde_json::json!({
                    "id": s.id,
                    "clientId": s.client_id,
                    "role": s.role,
                    "title": s.title,
                    "appId": s.app_id,
                    "size": s.size,
                    "position": s.position,
                    "sector": s.sector,
                    "activated": s.activated,
                    "committed": s.committed,
                })).collect::<Vec<_>>(),
                "compositorClients": compositor_clients,
                "gpuFrameCount": gpu_frame_count,
                "gpuVramUsedMb": gpu_vram_used_mb,
                "gpuVramBudgetMb": gpu_vram_budget_mb,
                "ptySessions": pty_sessions.iter().map(|s| serde_json::json!({
                    "surfaceId": s.surface_id,
                    "shellPath": s.shell_path,
                    "shellName": s.shell_name,
                    "cwd": s.cwd,
                })).collect::<Vec<_>>(),
                "zoomLevel": zoom_level,
                "redAlert": is_red_alert,
            })
        }
        BroadcastMessage::ClientViewport { width, height, device_pixel_ratio, dev_panel_open } => {
            serde_json::json!({
                "type": "client_viewport",
                "width": width,
                "height": height,
                "devicePixelRatio": device_pixel_ratio,
                "devPanelOpen": dev_panel_open,
            })
        }
    }
}

/// Handle for broadcasting messages to the dev monitor
#[derive(Clone)]
pub struct DevMonitorBroadcaster {
    tx: broadcast::Sender<BroadcastMessage>,
}

impl DevMonitorBroadcaster {
    pub fn send(&self, msg: BroadcastMessage) {
        let _ = self.tx.send(msg); // Ignore error if no receivers
    }

    // ─── Original convenience methods ──────────────

    pub fn update_viewport(&self, html: String, zoom: u8, red_alert: bool) {
        self.send(BroadcastMessage::UpdateViewport {
            html_content: html,
            zoom_level: zoom,
            is_red_alert: red_alert,
        });
    }

    pub fn update_dashboard(&self, html: String) {
        self.send(BroadcastMessage::UpdateDashboard(html));
    }

    pub fn zoom_level(&self, level: u8) {
        self.send(BroadcastMessage::ZoomLevel(level));
    }

    pub fn test_event(&self, test_name: impl Into<String>, event_type: impl Into<String>, details: impl Into<String>) {
        self.send(BroadcastMessage::TestEvent {
            test_name: test_name.into(),
            event_type: event_type.into(),
            details: details.into(),
        });
    }

    // ─── New infrastructure methods ────────────────

    /// Broadcast the current multi-viewport state
    pub fn send_viewport_state(&self, viewports: Vec<ViewportInfo>, focused_id: Option<u32>, output_count: usize) {
        self.send(BroadcastMessage::ViewportState {
            viewports,
            focused_id,
            output_count,
        });
    }

    /// Broadcast the current Wayland compositor state
    pub fn send_compositor_state(&self, surfaces: Vec<SurfaceInfo>, client_count: usize, focused_surface: Option<u64>, events: Vec<String>) {
        self.send(BroadcastMessage::CompositorState {
            surfaces,
            client_count,
            focused_surface,
            events,
        });
    }

    /// Broadcast GPU pipeline statistics
    pub fn send_gpu_stats(
        &self,
        frame_count: u64,
        texture_count: usize,
        vram_used_mb: f64,
        vram_budget_mb: f64,
        dirty_textures: usize,
        active_transition: Option<TransitionInfo>,
        fps: u32,
    ) {
        let vram_usage_pct = if vram_budget_mb > 0.0 {
            (vram_used_mb / vram_budget_mb) * 100.0
        } else { 0.0 };

        self.send(BroadcastMessage::GpuStats {
            frame_count,
            texture_count,
            vram_used_mb,
            vram_budget_mb,
            vram_usage_pct,
            dirty_textures,
            active_transition,
            fps,
        });
    }

    /// Broadcast PTY terminal output for a specific surface
    pub fn send_pty_output(&self, surface_id: u32, text: String) {
        self.send(BroadcastMessage::PtyOutput { surface_id, text });
    }

    /// Broadcast a PTY lifecycle event
    pub fn send_pty_event(&self, surface_id: u32, event_type: impl Into<String>, details: impl Into<String>) {
        self.send(BroadcastMessage::PtyEvent {
            surface_id,
            event_type: event_type.into(),
            details: details.into(),
        });
    }

    /// Broadcast PTY session listing
    pub fn send_pty_sessions(&self, sessions: Vec<PtySessionInfo>) {
        self.send(BroadcastMessage::PtySessionsState { sessions });
    }

    /// Send a full state snapshot (typically on initial client connect or periodic refresh)
    pub fn send_full_snapshot(
        &self,
        viewports: Vec<ViewportInfo>,
        focused_viewport: Option<u32>,
        surfaces: Vec<SurfaceInfo>,
        compositor_clients: usize,
        gpu_frame_count: u64,
        gpu_vram_used_mb: f64,
        gpu_vram_budget_mb: f64,
        pty_sessions: Vec<PtySessionInfo>,
        zoom_level: u8,
        is_red_alert: bool,
    ) {
        self.send(BroadcastMessage::FullSnapshot {
            viewports,
            focused_viewport,
            surfaces,
            compositor_clients,
            gpu_frame_count,
            gpu_vram_used_mb,
            gpu_vram_budget_mb,
            pty_sessions,
            zoom_level,
            is_red_alert,
        });
    }

    /// Broadcast a client viewport size change
    pub fn send_client_viewport(&self, width: u32, height: u32, dpr: f64, dev_panel_open: bool) {
        self.send(BroadcastMessage::ClientViewport {
            width,
            height,
            device_pixel_ratio: dpr,
            dev_panel_open,
        });
    }
}

/// Global dev monitor instance (optional)
static DEV_MONITOR: once_cell::sync::OnceCell<DevMonitorBroadcaster> = once_cell::sync::OnceCell::new();

/// Initialize the global dev monitor broadcaster
pub fn init_global_monitor(broadcaster: DevMonitorBroadcaster) {
    let _ = DEV_MONITOR.set(broadcaster);
}

/// Get the global dev monitor broadcaster (if initialized)
pub fn get_monitor() -> Option<&'static DevMonitorBroadcaster> {
    DEV_MONITOR.get()
}

// ─── Helper functions to extract state from TOS subsystems ──────

/// Extract ViewportInfo from the ViewportManager
pub fn extract_viewport_state(
    vm: &crate::navigation::viewport::ViewportManager,
) -> (Vec<ViewportInfo>, Option<u32>) {
    let mut infos = Vec::new();

    for vp in vm.all_viewports() {
        let output_name = vm.get_output(vp.output_id)
            .map(|o| o.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        infos.push(ViewportInfo {
            id: vp.id,
            label: vp.label.clone(),
            output_name,
            zoom_path: format!("{}", vp.path),
            zoom_depth: vp.path.depth(),
            current_level: format!("{:?}", vp.current_level),
            has_focus: vp.has_focus,
            geometry: [
                vp.geometry.x as f64,
                vp.geometry.y as f64,
                vp.geometry.width as f64,
                vp.geometry.height as f64,
            ],
        });
    }

    let focused_id = vm.focused_viewport_id();
    (infos, focused_id)
}

/// Extract SurfaceInfo from the WaylandBackend
pub fn extract_compositor_state(
    wb: &crate::compositor::wayland::WaylandBackend,
) -> (Vec<SurfaceInfo>, usize, Option<u64>) {
    let mut infos = Vec::new();

    for surface in wb.get_toplevels() {
        let (role_name, title, app_id, activated) = match &surface.role {
            Some(crate::compositor::wayland::SurfaceRole::Toplevel(state)) => {
                ("Toplevel".to_string(), state.title.clone(), state.app_id.clone(), state.activated)
            }
            Some(crate::compositor::wayland::SurfaceRole::XWayland(state)) => {
                ("XWayland".to_string(), state.title.clone(), state.class.clone(), false)
            }
            Some(crate::compositor::wayland::SurfaceRole::Popup { .. }) => {
                ("Popup".to_string(), String::new(), String::new(), false)
            }
            Some(crate::compositor::wayland::SurfaceRole::LcarsOverlay) => {
                ("LcarsOverlay".to_string(), String::new(), String::new(), false)
            }
            Some(crate::compositor::wayland::SurfaceRole::Subsurface { .. }) => {
                ("Subsurface".to_string(), String::new(), String::new(), false)
            }
            None => ("Unassigned".to_string(), String::new(), String::new(), false),
        };

        infos.push(SurfaceInfo {
            id: surface.id.0,
            client_id: surface.client_id.0,
            role: role_name,
            title,
            app_id,
            size: [surface.size.0, surface.size.1],
            position: [surface.position.0, surface.position.1],
            sector: surface.tos_sector,
            activated,
            committed: surface.committed,
        });
    }

    let focused = wb.get_focused_surface().map(|s| s.id.0);
    (infos, wb.client_count(), focused)
}

/// Extract GPU stats from the GpuPipeline
pub fn extract_gpu_stats(
    gpu: &crate::compositor::gpu::GpuPipeline,
) -> (u64, usize, f64, f64, f64, usize, Option<TransitionInfo>, u32) {
    let stats = gpu.cache_stats();
    let transition = gpu.current_transition.as_ref().map(|t| TransitionInfo {
        direction: if t.zooming_in { "zoom-in".to_string() } else { "zoom-out".to_string() },
        progress: t.progress(),
        duration_ms: t.duration_ms,
    });

    (
        gpu.frame_count,
        stats.texture_count,
        stats.vram_used_bytes as f64 / 1024.0 / 1024.0,
        stats.vram_budget_bytes as f64 / 1024.0 / 1024.0,
        stats.usage_percent(),
        stats.dirty_count,
        transition,
        gpu.target_fps,
    )
}
