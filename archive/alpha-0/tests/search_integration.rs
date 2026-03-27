use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::SurfaceRole;

#[test]
fn test_global_search_logic() {
    let mut env = DesktopEnvironment::new(None);
    
    // Create apps in different sectors
    env.surfaces.create_surface("Work Terminal", SurfaceRole::Toplevel, Some(0));
    env.surfaces.create_surface("Music Player", SurfaceRole::Toplevel, Some(1));
    env.surfaces.create_surface("System Monitor", SurfaceRole::Toplevel, Some(2));
    
    // 1. Level 1 with no search
    let html = env.generate_viewport_html();
    assert!(html.contains("WORK"));
    assert!(html.contains("MEDIA"));
    assert!(!html.contains("Work Terminal")); // Not visible at root unless searching
    
    // 2. Search for "Monitor"
    let _ = tos_comp::system::commands::CommandParser::process(&mut env, "find Monitor");
    let html = env.generate_viewport_html();
    
    assert!(html.contains("System Monitor"));
    assert!(html.contains("SECTOR: CORE")); // Sector 2 is Core
    assert!(!html.contains("Work Terminal"));
    assert!(!html.contains("ACTIVE APPS")); // Sector stats hidden during search
    
    // 3. Clear search
    let _ = tos_comp::system::commands::CommandParser::process(&mut env, "clear");
    let html = env.generate_viewport_html();
    assert!(html.contains("WORK"));
    assert!(!html.contains("System Monitor"));
}

#[test]
fn test_search_auto_resets_on_zoom() {
    let mut env = DesktopEnvironment::new(None);
    env.surfaces.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
    
    tos_comp::system::commands::CommandParser::process(&mut env, "find Term");
    assert!(env.search_query.is_some());
    
    // Zooming manually should clear search
    tos_comp::system::commands::CommandParser::process(&mut env, "zoom 2");
    assert!(env.search_query.is_none());
}
