// Visual tests - run with dev monitor to watch execution in browser
// cargo test --features dev-monitor --test visual_navigation -- --include-ignored

#![cfg(feature = "dev-monitor")]

mod visual_test_utils;
use visual_test_utils::VisualTestEnv;
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget};
use tos_comp::compositor::SurfaceRole;

#[test]
#[ignore]
fn visual_full_navigation_session() {
    let mut vt = VisualTestEnv::new("visual_full_navigation_session");
    
    vt.step("Initialize desktop environment");
    vt.env.dashboard.add_widget(Box::new(ClockWidget));
    vt.env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 15, ram_usage: 45 }));
    vt.update_dashboard();
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level1Root, "Should start at Level 1");

    vt.step("Create surfaces in different sectors");
    let s1 = vt.env.surfaces.create_surface("Terminal 1", SurfaceRole::Toplevel, Some(0));
    let s2 = vt.env.surfaces.create_surface("Terminal 2", SurfaceRole::Toplevel, Some(0));
    let s3 = vt.env.surfaces.create_surface("Browser", SurfaceRole::Toplevel, Some(1));
    vt.update_viewport();
    vt.assert(vt.env.surfaces.get_all_surface_titles().len() == 3, "Should have 3 surfaces");

    vt.step("Zoom into Work Sector (Level 2)");
    vt.env.navigator.zoom_in(0);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level2Sector, "Should be at Level 2");

    vt.step("Focus on Terminal 1 with multiple windows (should go to picker)");
    vt.env.navigator.zoom_in(0); // Even index = multi window
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level3aPicker, "Should be at picker");

    vt.step("Select specific window from picker");
    vt.env.navigator.zoom_in(0); // Select first window
    vt.env.navigator.active_app_index = Some(0);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level3Focus, "Should be at focus");

    vt.step("Zoom into detail view (Level 4)");
    vt.env.navigator.zoom_in(0);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level4Detail, "Should be at detail");

    vt.step("Access raw buffer (Level 5)");
    vt.env.navigator.zoom_in(0);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level5Buffer, "Should be at buffer");

    vt.step("Zoom back out to detail");
    vt.env.navigator.zoom_out(false);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level4Detail, "Should be back at detail");

    vt.step("Intelligent zoom out (multi-window = picker)");
    vt.env.intelligent_zoom_out();
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level3aPicker, "Should be at picker");

    vt.step("Zoom out to sector");
    vt.env.navigator.zoom_out(false);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level2Sector, "Should be at sector");

    vt.step("Zoom out to root");
    vt.env.navigator.zoom_out(false);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level1Root, "Should be at root");

    vt.step("Update system stats via tick");
    vt.env.tick();
    vt.update_dashboard();
    vt.update_viewport();

    vt.finish();
}

#[test]
#[ignore]
fn visual_split_view_test() {
    let mut vt = VisualTestEnv::new("visual_split_view_test");
    
    vt.step("Setup environment with surfaces");
    vt.env.dashboard.add_widget(Box::new(ClockWidget));
    let s1 = vt.env.surfaces.create_surface("Editor", SurfaceRole::Toplevel, Some(0));
    let s2 = vt.env.surfaces.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
    vt.update_viewport();

    vt.step("Navigate to sector");
    vt.env.navigator.zoom_in(0);
    vt.update_viewport();

    vt.step("Focus on Editor");
    vt.env.navigator.zoom_in(0);
    vt.env.navigator.active_app_index = Some(0);
    vt.update_viewport();

    vt.step("Enter split view with Terminal");
    vt.env.navigator.split_view(s2);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level3Split, "Should be in split view");
    vt.assert(vt.env.navigator.secondary_app_id == Some(s2), "Secondary should be Terminal");

    vt.step("Swap split panels");
    let swapped = vt.env.swap_split();
    vt.update_viewport();
    vt.assert(swapped, "Swap should succeed");
    vt.assert(vt.env.navigator.secondary_app_id == Some(s1), "Secondary should now be Editor");

    vt.step("Exit split view");
    vt.env.navigator.zoom_out(false);
    vt.update_viewport();
    vt.assert(vt.env.navigator.current_level == tos_comp::navigation::zoom::ZoomLevel::Level3Focus, "Should be back at focus");

    vt.finish();
}

#[test]
#[ignore]
fn visual_red_alert_test() {
    let mut vt = VisualTestEnv::new("visual_red_alert_test");
    
    vt.step("Setup normal state");
    vt.env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 20, ram_usage: 30 }));
    vt.update_viewport();
    vt.assert(!vt.env.is_red_alert, "Should not be in red alert");

    vt.step("Add critical notification");
    vt.env.notifications.push("CRITICAL", "Core breach detected!", tos_comp::system::notifications::Priority::Critical);
    vt.update_viewport();

    vt.step("Tick to process notifications");
    vt.env.tick();
    vt.update_viewport();
    vt.assert(vt.env.is_red_alert, "Should be in red alert mode");

    vt.step("Clear notification");
    vt.env.notifications.queue.clear();
    vt.env.tick();
    vt.update_viewport();
    vt.assert(!vt.env.is_red_alert, "Red alert should be cleared");

    vt.finish();
}
