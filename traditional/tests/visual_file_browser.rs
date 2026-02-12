// Visual test: File Browser with ls command and context menu
// Run: cargo test --features dev-monitor --test visual_file_browser -- --include-ignored
//
// Steps:
//   1. Setup environment with a File Manager surface
//   2. Simulate "ls" — populate the VFS with realistic entries
//   3. Navigate into the sector → focus on File Manager
//   4. Simulate long-press/right-click on first item (context menu)
//   5. Wait for viewer, then dismiss

#![cfg(feature = "dev-monitor")]

mod visual_test_utils;
use visual_test_utils::VisualTestEnv;
use tos_comp::compositor::SurfaceRole;
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget};
use tos_comp::dev_monitor::get_monitor;

#[test]
#[ignore]
fn visual_ls_and_context_menu() {
    let mut vt = VisualTestEnv::new("visual_ls_and_context_menu");

    // ── Step 1: Initialize the desktop ──────────────────────────

    vt.step_slow("Initialize desktop with dashboard widgets", 5);
    vt.env.dashboard.add_widget(Box::new(ClockWidget));
    vt.env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 12, ram_usage: 38 }));
    vt.update_dashboard();

    // Create some surfaces in different sectors
    let _terminal = vt.env.surfaces.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
    let _file_mgr = vt.env.surfaces.create_surface("File Manager", SurfaceRole::Toplevel, Some(0));
    let _browser = vt.env.surfaces.create_surface("Browser", SurfaceRole::Toplevel, Some(1));

    vt.update_viewport();
    vt.assert(
        vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level1Root,
        "Should start at root overview (Level 1)",
    );

    // ── Step 2: Simulate "ls" — add files to VFS ────────────────

    vt.step_slow("Simulating 'ls' — populating /home/user with file entries", 5);

    // The VFS starts at /home/user with {documents/, notes.txt}
    // Add more realistic entries as if the user ran `ls`
    vt.env.files.create_file("README.md");
    vt.env.files.create_file("Cargo.toml");
    vt.env.files.create_file("main.rs");
    vt.env.files.create_dir("src");
    vt.env.files.create_dir(".config");
    vt.env.files.create_file("todo.txt");
    vt.env.files.create_file("screenshot.png");

    // Verify entries were added
    let entry_count = vt.env.files.get_current_entries().map(|e| e.len()).unwrap_or(0);
    vt.assert(entry_count >= 7, &format!("Should have at least 7 entries after ls, got {}", entry_count));

    // Update the viewport to show root with the new files available
    vt.update_viewport();

    // ── Step 3: Navigate into Work sector ──────────────────────

    vt.step_slow("Zooming into Work sector (Level 2)", 5);
    vt.env.navigator.zoom_in(0); // Sector 0 = Work
    vt.update_viewport();
    vt.assert(
        vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level2Sector,
        "Should be at sector view (Level 2)",
    );

    // ── Step 4: Focus on File Manager ──────────────────────────

    vt.step_slow("Focusing on File Manager (Level 3) — file listing visible", 5);
    // Work sector has [Terminal, File Manager] — File Manager is at index 1
    // With 2 surfaces in a sector, zoom_in goes to picker first (Level3aPicker)
    vt.env.navigator.zoom_in(0); // Enter the apps
    vt.update_viewport();

    // If we're at picker because of multiple surfaces, select the File Manager
    if vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level3aPicker {
        vt.step_slow("Selecting File Manager from picker", 5);
        vt.env.navigator.zoom_in(1); // Select second item (File Manager)
        vt.env.navigator.active_app_index = Some(1);
        vt.update_viewport();
    } else {
        // Direct focus — set index to File Manager
        vt.env.navigator.active_app_index = Some(1);
        vt.update_viewport();
    }

    vt.assert(
        vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level3Focus,
        "Should be focused on File Manager (Level 3)",
    );

    // ── Step 5: Long-press/right-click on first file entry ─────

    vt.step_slow("Long-press / right-click on first file item — showing context menu", 5);

    // The context menu is purely client-side (JS handles the right-click).
    // For the visual test, we inject a test event that tells the browser to
    // simulate the context menu opening on the first .file-item element.
    if let Some(monitor) = get_monitor() {
        // Send a test step event that the frontend can use
        monitor.test_event(
            "visual_ls_and_context_menu",
            "action",
            "context_menu:first:.file-item",
        );
    }

    // Wait 5 seconds for the viewer to see the context menu
    std::thread::sleep(std::time::Duration::from_secs(5));

    // ── Step 6: Dismiss context menu ───────────────────────────

    vt.step_slow("Dismissing context menu", 5);
    if let Some(monitor) = get_monitor() {
        monitor.test_event(
            "visual_ls_and_context_menu",
            "action",
            "context_menu:dismiss",
        );
    }

    // ── Step 7: Navigate back to root ──────────────────────────

    vt.step_slow("Zooming back out to root", 5);
    vt.env.navigator.zoom_out(false); // Back to sector or picker
    vt.update_viewport();
    vt.env.navigator.zoom_out(false); // Back to sector
    vt.update_viewport();
    vt.env.navigator.zoom_out(false); // Back to root
    vt.update_viewport();
    vt.assert(
        vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level1Root,
        "Should be back at root (Level 1)",
    );

    vt.finish();
}
