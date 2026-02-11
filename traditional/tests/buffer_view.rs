use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::SurfaceRole;

#[test]
fn test_hex_dump_generation() {
    let mut env = DesktopEnvironment::new(None);
    let _sid = env.surfaces.create_surface("Kernel Core", SurfaceRole::Toplevel, Some(0));
    
    // Zoom all the way to Level 5
    env.navigator.zoom_in(0); // Level 2
    env.navigator.zoom_in(0); // Level 3 (Focus)
    env.navigator.zoom_in(0); // Level 4 (Detail)
    env.navigator.zoom_in(0); // Level 5 (Buffer)
    
    assert_eq!(env.navigator.current_level, ZoomLevel::Level5Buffer);
    
    let html = env.generate_viewport_html();
    
    // Verify structural elements
    assert!(html.contains("RAW MEMORY BUFFER"));
    assert!(html.contains("hex-addr"));
    assert!(html.contains("hex-data"));
    // Verify we have multiple lines
    assert!(html.matches("hex-line").count() >= 10);
    
    // Zoom out
    env.intelligent_zoom_out();
    assert_eq!(env.navigator.current_level, ZoomLevel::Level4Detail);
}
