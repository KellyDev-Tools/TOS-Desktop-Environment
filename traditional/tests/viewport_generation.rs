use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::SurfaceRole;
use tos_comp::ui::dashboard::ClockWidget;

#[test]
fn test_viewport_generation_logic() {
    let mut env = DesktopEnvironment::new(None);
    env.dashboard.add_widget(Box::new(ClockWidget));
    
    // Level 1: Root
    let html = env.generate_viewport_html();
    assert!(html.contains("dashboard-layer"));
    assert!(html.contains("CLOCK"));
    assert!(html.contains("surfaces-grid"));
    assert!(!html.contains("lcars-window-frame")); // No surfaces yet

    // Level 2: Zoom into Sector 0
    let _term = env.surfaces.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
    env.navigator.zoom_in(0);
    
    let html = env.generate_viewport_html();
    assert!(html.contains("dashboard-layer"));
    assert!(html.contains("Terminal"));
    assert!(html.contains("lcars-window-frame"));

    // Level 3: Focus
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);
}

#[test]
fn test_viewport_morphing() {
    let mut env = DesktopEnvironment::new(None);
    env.surfaces.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
    env.navigator.zoom_in(0); // Level 2

    // Default is Static
    let html = env.generate_viewport_html();
    assert!(html.contains("morph-static"));

    // Start Entering morph
    env.start_zoom_morph(true);
    let html = env.generate_viewport_html();
    assert!(html.contains("morph-entering"));

    // Finish morph
    env.finish_morph();
    let html = env.generate_viewport_html();
    assert!(html.contains("morph-static"));
    assert!(!html.contains("morph-entering"));
}
