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
    assert!(html.contains("lcars-window-frame")); // Sectors are rendered as frames
    assert!(html.contains("WORK")); // One of the default sectors

    // Level 2: Zoom into Sector 0
    let _term = env.surfaces.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
    env.navigator.zoom_in(0);
    
    let html = env.generate_viewport_html();
    assert!(html.contains("dashboard-layer"));
    assert!(html.contains("Terminal"));
    assert!(html.contains("lcars-window-frame"));

    // Level 3: Focus
    env.navigator.zoom_in(0);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Focus);

    // Level 4: Detail
    env.navigator.zoom_in(0);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level4Detail);
    let html = env.generate_viewport_html();
    assert!(html.contains("NODE HISTORY"));
    assert!(html.contains("Surface created: Terminal"));
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

#[test]
fn test_viewport_picker_and_split() {
    let mut env = DesktopEnvironment::new(None);
    let _s1 = env.surfaces.create_surface("Term 1", SurfaceRole::Toplevel, Some(0));
    let s2 = env.surfaces.create_surface("Term 2", SurfaceRole::Toplevel, Some(0));
    
    // 1. Split View
    env.navigator.zoom_in(0); // Sector
    env.navigator.zoom_in(0); // Focus s1
    env.navigator.split_view(s2);
    
    let html_split = env.generate_viewport_html();
    assert!(html_split.contains("Term 1"));
    assert!(html_split.contains("Term 2"));
    assert!(html_split.contains("SWAP SLOTS")); // Special button for split
    
    // 2. Picker View
    env.navigator.zoom_out(true); // Split -> Focus
    env.navigator.zoom_out(true); // Focus -> Picker (since 2 apps exist)
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3aPicker);
    
    let html_picker = env.generate_viewport_html();
    assert!(html_picker.contains("Term 1"));
    assert!(html_picker.contains("Term 2"));
    assert!(html_picker.contains("zoom:3:0")); // Picker buttons use index
}
