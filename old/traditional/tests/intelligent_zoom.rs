use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::SurfaceRole;

#[test]
fn test_intelligent_zoom_out_picker_trigger() {
    let mut env = DesktopEnvironment::new(None);
    
    // Group 1: 2 apps
    env.surfaces.create_surface("Term 1", SurfaceRole::Toplevel, Some(0));
    env.surfaces.create_surface("Term 2", SurfaceRole::Toplevel, Some(0));
    
    // Group 2: 1 app
    env.surfaces.create_surface("Browser", SurfaceRole::Toplevel, Some(1));
    
    // 1. Focus on Term 1
    env.navigator.zoom_in(0); // Sector 0
    env.navigator.zoom_in(0); // Focus Term 1 (index 0)
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Focus);
    
    // Intelligent Zoom Out from Group with 2 apps -> Picker
    env.intelligent_zoom_out();
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3aPicker);
    
    // 2. Focus on Browser
    env.navigator.zoom_out(false); // Back to sector
    env.navigator.zoom_out(false); // Back to root
    env.navigator.zoom_in(1); // Sector 1
    env.navigator.zoom_in(0); // Focus Browser
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Focus);
    
    // Intelligent Zoom Out from Group with 1 app -> Sector
    env.intelligent_zoom_out();
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);
}

#[test]
fn test_intelligent_zoom_out_from_detail() {
    let mut env = DesktopEnvironment::new(None);
    env.surfaces.create_surface("App", SurfaceRole::Toplevel, Some(0));
    
    env.navigator.zoom_in(0); // Sector
    env.navigator.zoom_in(0); // Focus
    env.navigator.zoom_in(0); // Detail
    
    assert_eq!(env.navigator.current_level, ZoomLevel::Level4Detail);
    
    // Zoom out from detail should go to Focus
    env.intelligent_zoom_out();
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Focus);
}

#[test]
fn test_intelligent_zoom_out_from_root() {
    let mut env = DesktopEnvironment::new(None);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level1Root);
    
    // Should stay at root and not panic
    env.intelligent_zoom_out();
    assert_eq!(env.navigator.current_level, ZoomLevel::Level1Root);
}
