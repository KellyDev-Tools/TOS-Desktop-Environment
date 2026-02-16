use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::{SurfaceRole, SpatialMapper};
use tos_comp::system::notifications::Priority;

#[test]
fn test_spatial_integrity_and_overflow() {
    let mut env = DesktopEnvironment::new(None);
    for i in 0..10 {
        env.surfaces.create_surface(&format!("App {}", i), SurfaceRole::Toplevel, Some(0));
    }
    let layouts = SpatialMapper::get_layout(&env.surfaces, ZoomLevel::Level2Sector, Some(0), None, None, 4);
    assert_eq!(layouts.len(), 10);
    // Verify incrementing grid (Row 3 for App 6)
    assert_eq!(layouts[6].grid_x, 0);
    assert_eq!(layouts[6].grid_y, 3); 
}

#[test]
fn test_global_search_cross_domain() {
    let mut env = DesktopEnvironment::new(None);
    env.surfaces.create_surface("Logic Terminal", SurfaceRole::Toplevel, Some(0));
    env.files.create_file("logic_notes.txt");
    env.search_query = Some("logic".to_string());
    let html = env.generate_viewport_html();
    assert!(html.contains("Logic Terminal"));
    assert!(html.contains("logic_notes.txt"));
}

#[test]
fn test_red_alert_logic() {
    let mut env = DesktopEnvironment::new(None);
    env.notifications.push("DANGER", "Core breach", Priority::Critical);
    env.tick();
    assert!(env.is_red_alert);
    env.notifications.process_next();
    env.tick();
    assert!(!env.is_red_alert);
}

#[test]
fn test_shell_osc_sync() {
    let mut env = DesktopEnvironment::new(None);
    env.handle_shell_output("\x1b]1337;ZoomLevel=2\x07");
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);
    
    env.handle_shell_output("\x1b]1337;CurrentDir=/tmp/work\x07");
    assert_eq!(env.files.current_path, "/tmp/work");
}
