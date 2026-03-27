// Demo backend that exercises the full TOS infrastructure with live monitoring
//
// This creates a simulated compositor session with:
// - Multi-viewport management (split panes, independent zoom)
// - Wayland surface lifecycle (create, focus, sector assignment)
// - GPU pipeline stats (texture cache, VRAM, transitions)
// - PTY session tracking
//
// Usage: cargo run --features dev-monitor --bin demo-backend [port]

#![cfg(feature = "dev-monitor")]

use tos_comp::{DesktopEnvironment, dev_monitor::*};
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget, ProcessManagerWidget};
use tos_comp::compositor::{SurfaceRole as LegacySurfaceRole};
use tos_comp::compositor::wayland::WaylandBackend;
use tos_comp::compositor::gpu::{GpuPipeline, TextureFormat, NormRect};
use tos_comp::navigation::viewport::{ZoomPath};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(3000);

    println!("{}", "═".repeat(62));
    println!("  ████████╗ ██████╗ ███████╗    DEMO");
    println!("  ╚══██╔══╝██╔═══██╗██╔════╝    Development Monitor");
    println!("     ██║   ██║   ██║███████╗    Full Infrastructure");
    println!("     ██║   ██║   ██║╚════██║");
    println!("     ██║   ╚██████╔╝███████║");
    println!("     ╚═╝    ╚═════╝ ╚══════╝");
    println!("{}", "═".repeat(62));
    println!();
    println!("  ┌─────────────────────────────────────────────────┐");
    println!("  │  1. Open http://127.0.0.1:{:<5} in your browser  │", port);
    println!("  │  2. Watch all four subsystems come alive!       │");
    println!("  │                                                 │");
    println!("  │  Subsystems:                                    │");
    println!("  │    ◆ Multi-Viewport Manager                     │");
    println!("  │    ◆ Wayland Compositor Backend                 │");
    println!("  │    ◆ GPU Rendering Pipeline                     │");
    println!("  │    ◆ PTY Shell Sessions                         │");
    println!("  └─────────────────────────────────────────────────┘");
    println!();

    // ─── Start Dev Monitor Server ──────────────────
    let monitor = DevMonitor::new(port);
    let broadcaster = monitor.get_broadcaster();
    init_global_monitor(broadcaster);

    tokio::spawn(async move {
        if let Err(e) = monitor.run().await {
            eprintln!("[Error] Server: {}", e);
        }
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    // ─── Initialize Subsystems ─────────────────────
    println!("[Init] Creating Desktop Environment...");
    let mut env = DesktopEnvironment::new(None);

    // Dashboard widgets
    env.dashboard.add_widget(Box::new(ClockWidget));
    env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 15, ram_usage: 45 }));
    env.dashboard.add_widget(Box::new(ProcessManagerWidget { processes: vec![] }));

    // Legacy surfaces (for HTML viewport)
    env.surfaces.create_surface("Terminal", LegacySurfaceRole::Toplevel, Some(0));
    env.surfaces.create_surface("Browser", LegacySurfaceRole::Toplevel, Some(1));
    env.surfaces.create_surface("File Manager", LegacySurfaceRole::Toplevel, Some(0));
    env.surfaces.create_surface("Editor", LegacySurfaceRole::Toplevel, Some(2));

    println!("[Init] Creating Wayland Backend...");
    let mut wayland = WaylandBackend::new();

    // Simulate Wayland clients connecting and creating surfaces
    let client1 = wayland.client_connect();
    let ws1 = wayland.create_surface(client1);
    wayland.assign_toplevel_role(ws1, "org.gnome.Terminal", "Terminal — ~");
    wayland.commit_surface(ws1, 1200, 800);
    wayland.assign_to_sector(ws1, 0);
    wayland.configure_surface(ws1, 0, 40, 1200, 800);

    let client2 = wayland.client_connect();
    let ws2 = wayland.create_surface(client2);
    wayland.assign_toplevel_role(ws2, "org.mozilla.firefox", "Firefox — Wayland Research");
    wayland.commit_surface(ws2, 1920, 1040);
    wayland.assign_to_sector(ws2, 1);
    wayland.configure_surface(ws2, 0, 40, 1920, 1040);

    let ws3 = wayland.create_surface(client1);
    wayland.assign_toplevel_role(ws3, "org.gnome.Nautilus", "Files — /home/user");
    wayland.commit_surface(ws3, 800, 600);
    wayland.assign_to_sector(ws3, 0);
    wayland.configure_surface(ws3, 0, 40, 800, 600);

    let client3 = wayland.client_connect();
    let ws4 = wayland.create_surface(client3);
    wayland.map_xwayland_window(ws4, 0x4001, "code", "Visual Studio Code");
    wayland.commit_surface(ws4, 1600, 900);
    wayland.assign_to_sector(ws4, 2);
    wayland.configure_surface(ws4, 0, 40, 1600, 900);

    wayland.set_keyboard_focus(Some(ws1));

    println!("[Init] Creating GPU Pipeline...");
    let mut gpu = GpuPipeline::new(256, 60);

    // Pre-allocate textures for the surfaces
    let tex1 = gpu.texture_cache.allocate(1200, 800, TextureFormat::Bgra8Unorm, 2);
    let _tex2 = gpu.texture_cache.allocate(1920, 1040, TextureFormat::Bgra8Unorm, 1);
    let _tex3 = gpu.texture_cache.allocate(800, 600, TextureFormat::Bgra8Unorm, 1);
    let _tex4 = gpu.texture_cache.allocate(1600, 900, TextureFormat::Bgra8Unorm, 2);
    // LCARS chrome overlay
    let _chrome = gpu.texture_cache.allocate(1920, 1080, TextureFormat::Bgra8Unorm, 0);

    println!("[Init] Setting up ViewportManager...");
    // The DesktopEnvironment already has a ViewportManager, use that
    let vm = &mut env.viewport_manager;

    println!();
    println!("✓ All subsystems initialized!");
    println!("  Wayland: {} clients, {} surfaces", wayland.client_count(), wayland.surface_count());
    println!("  GPU: {:.1}MB VRAM allocated", gpu.cache_stats().vram_used_bytes as f64 / 1024.0 / 1024.0);
    println!("  Viewports: {}", vm.viewport_count());
    println!();

    // ─── Send Initial Snapshot ─────────────────────
    send_all_state(&mut env, &wayland, &gpu);
    send_viewport_html(&mut env);

    println!("Starting demo sequence in 3 seconds...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // ═══════════════════════════════════════════════
    // DEMO SEQUENCE
    // ═══════════════════════════════════════════════

    let bcast = get_monitor().unwrap();

    // ─── Phase 1: Zoom Navigation ──────────────────
    bcast.test_event("infrastructure_demo", "step", "Phase 1: Zoom Navigation");
    println!("\n═══ Phase 1: Zoom Navigation ═══\n");

    println!("  → Zooming to Work Sector (Level 2)");
    env.navigator.zoom_in(0);
    send_viewport_html(&mut env);
    gpu.start_transition(NormRect::full(), NormRect::tile(0, 0, 2, 2), true);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Focusing on Terminal (Level 3)");
    env.navigator.zoom_in(0);
    env.navigator.active_app_index = Some(0);
    send_viewport_html(&mut env);
    gpu.start_transition(NormRect::tile(0, 0, 2, 2), NormRect::full(), true);
    wayland.set_keyboard_focus(Some(ws1));
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Opening Detail View (Level 4)");
    env.navigator.zoom_in(0);
    send_viewport_html(&mut env);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Back to Root");
    env.navigator.zoom_out(false);
    env.navigator.zoom_out(false);
    env.navigator.zoom_out(false);
    send_viewport_html(&mut env);
    gpu.start_transition(NormRect::tile(0, 0, 2, 2), NormRect::full(), false);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    // ─── Phase 2: Viewport Splits ──────────────────
    bcast.test_event("infrastructure_demo", "step", "Phase 2: Viewport Splits");
    println!("\n═══ Phase 2: Viewport Splits ═══\n");

    let primary_vp = env.viewport_manager.focused_viewport_id().unwrap();
    println!("  → Splitting primary viewport horizontally");
    if let Some(new_vp) = env.viewport_manager.split_horizontal(primary_vp) {
        println!("    Created viewport {} (Split-Right)", new_vp);
    }
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Zooming left pane to Sector 0");
    env.viewport_manager.zoom_in_focused(0);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Switching focus to right pane");
    let viewports: Vec<_> = env.viewport_manager.all_viewports().iter().map(|v| v.id).collect();
    if viewports.len() > 1 {
        env.viewport_manager.set_focus(viewports[1]);
    }
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Zooming right pane to Sector 1 (independent!)");
    env.viewport_manager.zoom_in_focused(1);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Unsplitting — back to single viewport");
    env.viewport_manager.unsplit(primary_vp);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    // ─── Phase 3: Compositor Lifecycle ─────────────
    bcast.test_event("infrastructure_demo", "step", "Phase 3: Compositor Lifecycle");
    println!("\n═══ Phase 3: Compositor Lifecycle ═══\n");

    println!("  → New client connecting...");
    let client4 = wayland.client_connect();
    let ws5 = wayland.create_surface(client4);
    wayland.assign_toplevel_role(ws5, "org.kde.dolphin", "Dolphin File Manager");
    wayland.commit_surface(ws5, 1024, 768);
    wayland.assign_to_sector(ws5, 0);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Window title change...");
    wayland.set_toplevel_title(ws1, "Terminal — ~/projects/tos");
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("  → Focus cycling: Terminal → Firefox");
    wayland.set_keyboard_focus(Some(ws2));
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("  → Focus cycling: Firefox → VS Code (XWayland)");
    wayland.set_keyboard_focus(Some(ws4));
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("  → Client disconnecting (Dolphin)...");
    wayland.client_disconnect(client4);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    // ─── Phase 4: GPU Activity ─────────────────────
    bcast.test_event("infrastructure_demo", "step", "Phase 4: GPU Activity");
    println!("\n═══ Phase 4: GPU Activity ═══\n");

    println!("  → Allocating additional textures...");
    for i in 0..5 {
        let _t = gpu.texture_cache.allocate(512, 512, TextureFormat::Bgra8Unorm, i % 4);
        send_all_state(&mut env, &wayland, &gpu);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!("  → Pruning deep textures (depth > 2)...");
    gpu.texture_cache.prune_below_depth(2);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("  → Simulating zoom transition...");
    gpu.start_transition(NormRect::full(), NormRect::tile(0, 0, 2, 2), true);

    for _ in 0..20 {
        let surfaces = vec![
            (tex1, NormRect::full(), 1.0f32),
        ];
        let _frame = gpu.build_frame(&surfaces, None, NormRect::full());
        gpu.end_frame();
        send_all_state(&mut env, &wayland, &gpu);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    tokio::time::sleep(Duration::from_secs(1)).await;

    // ─── Phase 5: Red Alert ────────────────────────
    bcast.test_event("infrastructure_demo", "step", "Phase 5: Red Alert & Recovery");
    println!("\n═══ Phase 5: Red Alert ═══\n");

    println!("  → Triggering Red Alert!");
    env.notifications.push("WARP CORE", "Core breach detected — immediate action required!", tos_comp::system::notifications::Priority::Critical);
    env.tick();
    send_viewport_html(&mut env);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("  → Clearing alert...");
    env.notifications.queue.clear();
    env.tick();
    send_viewport_html(&mut env);
    send_all_state(&mut env, &wayland, &gpu);
    tokio::time::sleep(Duration::from_secs(2)).await;

    // ─── Continuous Loop ───────────────────────────
    bcast.test_event("infrastructure_demo", "completed", "✓ Demo sequence complete — entering live mode");
    println!();
    println!("═══ Demo Complete! Entering live monitoring mode ═══");
    println!("    Server running on http://127.0.0.1:{}", port);
    println!("    Press Ctrl+C to exit");
    println!();

    let mut tick_count = 0u64;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        tick_count += 1;

        env.tick();

        // Simulate some GPU work
        let surfaces = vec![(tex1, NormRect::full(), 1.0f32)];
        let _frame = gpu.build_frame(&surfaces, None, NormRect::full());
        gpu.end_frame();

        // Periodic state broadcast
        if tick_count % 2 == 0 {
            send_all_state(&mut env, &wayland, &gpu);
            send_viewport_html(&mut env);
        }
    }
}

/// Send viewport HTML update
fn send_viewport_html(env: &mut DesktopEnvironment) {
    let html = env.generate_viewport_html();
    let zoom: u8 = match env.navigator.current_level {
        tos_comp::navigation::zoom::ZoomLevel::Level1Root => 1,
        tos_comp::navigation::zoom::ZoomLevel::Level2Sector => 2,
        tos_comp::navigation::zoom::ZoomLevel::Level3Focus => 3,
        tos_comp::navigation::zoom::ZoomLevel::Level3aPicker => 3,
        tos_comp::navigation::zoom::ZoomLevel::Level3Split => 3,
        tos_comp::navigation::zoom::ZoomLevel::Level4Detail => 4,
        tos_comp::navigation::zoom::ZoomLevel::Level5Buffer => 5,
    };

    if let Some(monitor) = get_monitor() {
        monitor.update_viewport(html, zoom, env.is_red_alert);
    }
}

/// Send all subsystem state to the dev monitor
fn send_all_state(
    env: &mut DesktopEnvironment,
    wayland: &WaylandBackend,
    gpu: &GpuPipeline,
) {
    let monitor = match get_monitor() {
        Some(m) => m,
        None => return,
    };

    // Viewport state
    let (vp_infos, focused_vp) = extract_viewport_state(&env.viewport_manager);
    let output_count = env.viewport_manager.output_count();
    monitor.send_viewport_state(vp_infos, focused_vp, output_count);

    // Compositor state
    let (surface_infos, client_count, focused_surface) = extract_compositor_state(wayland);
    monitor.send_compositor_state(surface_infos, client_count, focused_surface, vec![]);

    // GPU stats
    let (frame_count, tex_count, vram_used, vram_budget, _pct, dirty, transition, fps) = extract_gpu_stats(gpu);
    monitor.send_gpu_stats(frame_count, tex_count, vram_used, vram_budget, dirty, transition, fps);
}
