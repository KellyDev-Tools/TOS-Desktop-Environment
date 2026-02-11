// Demo backend that runs with dev monitor
// This shows the TOS UI actually working in the browser

#![cfg(feature = "dev-monitor")]

use tos_comp::{DesktopEnvironment, dev_monitor::{DevMonitor, init_global_monitor, get_monitor}};
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget, ProcessManagerWidget};
use tos_comp::compositor::SurfaceRole;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(3000);

    println!("{}", "=".repeat(60));
    println!("  TOS Demo Backend with Live Monitor");
    println!("{}", "=".repeat(60));
    println!();
    println!("  1. Open http://127.0.0.1:{} in your browser", port);
    println!("  2. Watch the TOS interface come alive!");
    println!();
    println!("{}", "=".repeat(60));
    println!();

    // Start dev monitor server
    let monitor = DevMonitor::new(port);
    let broadcaster = monitor.get_broadcaster();
    init_global_monitor(broadcaster);

    // Spawn the server in background
    tokio::spawn(async move {
        if let Err(e) = monitor.run().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("Starting TOS Desktop Environment...");
    
    // Create environment
    let mut env = DesktopEnvironment::new(None);
    
    // Add dashboard widgets
    env.dashboard.add_widget(Box::new(ClockWidget));
    env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 15, ram_usage: 45 }));
    env.dashboard.add_widget(Box::new(ProcessManagerWidget { processes: vec![] }));

    // Create some surfaces
    println!("Creating surfaces...");
    env.surfaces.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
    env.surfaces.create_surface("Browser", SurfaceRole::Toplevel, Some(1));
    env.surfaces.create_surface("File Manager", SurfaceRole::Toplevel, Some(0));
    env.surfaces.create_surface("Editor", SurfaceRole::Toplevel, Some(2));

    // Initial viewport
    send_viewport(&mut env);
    
    println!();
    println!("✓ Browser should now show the TOS interface!");
    println!();
    println!("Demo sequence starting in 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Demo sequence
    println!("\n=== Starting Navigation Demo ===\n");
    
    for i in 0..5 {
        println!("Demo cycle {}...", i + 1);
        
        // Zoom to sector
        println!("  → Zooming to Work Sector (Level 2)");
        env.navigator.zoom_in(0);
        send_viewport(&mut env);
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Zoom to app
        println!("  → Focusing on Terminal (Level 3)");
        env.navigator.zoom_in(0);
        env.navigator.active_app_index = Some(0);
        send_viewport(&mut env);
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Zoom to detail
        println!("  → Opening Detail View (Level 4)");
        env.navigator.zoom_in(0);
        send_viewport(&mut env);
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Zoom back out
        println!("  → Zooming back to Sector");
        env.navigator.zoom_out(false);
        env.navigator.zoom_out(false);
        send_viewport(&mut env);
        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("  → Returning to Root");
        env.navigator.zoom_out(false);
        send_viewport(&mut env);
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Update system stats
        env.tick();
        send_viewport(&mut env);
    }

    println!();
    println!("=== Demo Complete! Server still running. Press Ctrl+C to exit. ===");
    println!();

    // Keep running
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        env.tick();
        if get_monitor().is_some() {
            send_viewport(&mut env);
        }
    }
}

fn send_viewport(env: &mut DesktopEnvironment) {
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
