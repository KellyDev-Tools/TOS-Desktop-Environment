use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::{SurfaceRole, SpatialMapper};

#[test]
fn test_surface_mapping_integration() {
    let mut env = DesktopEnvironment::new(None);
    
    // Create some surfaces across different sectors
    let term = env.surfaces.create_surface("Work Terminal", SurfaceRole::Toplevel, Some(0)); // Sector 0
    let browser = env.surfaces.create_surface("Music Player", SurfaceRole::Toplevel, Some(1)); // Sector 1
    
    // Level 1: Root
    let visible = SpatialMapper::get_visible_surfaces(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        None
    );
    assert_eq!(visible.len(), 0); // Root overview doesnt show apps directly

    // Level 2: Zoom into Work Sector (0)
    env.navigator.zoom_in(0);
    let visible = SpatialMapper::get_visible_surfaces(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        None
    );
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].title, "Work Terminal");

    // Level 3: Focus on Terminal
    env.navigator.zoom_in(0); // target index doesn't strictly matter for focus in this mock yet
    let visible = SpatialMapper::get_visible_surfaces(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        Some(term)
    );
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, term);
}

#[test]
fn test_picker_mapping() {
    let mut env = DesktopEnvironment::new(None);
    
    // Add multiple windows to the same app/sector
    let w1 = env.surfaces.create_surface("Term Window 1", SurfaceRole::Toplevel, Some(0));
    let _w2 = env.surfaces.create_surface("Term Window 2", SurfaceRole::Toplevel, Some(0));
    
    // Zoom into Sector 0
    env.navigator.zoom_in(0);
    
    // Navigator logic: if we have multiple windows, zoom_out from Focus (L3) should go to Picker (L3a)
    // First, focus on window 1
    env.navigator.zoom_in(0); // Now at L3
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Focus);
    
    // zoom_out should go to Picker if app index is even (mock logic)
    env.navigator.active_app_index = Some(0); // Even index app
    env.navigator.zoom_out();
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3aPicker);
    
    // In Picker, we show all windows for that app/sector
    let visible = SpatialMapper::get_visible_surfaces(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        Some(w1) // in Picker, last focused is often shown or picker list is used
    );
    assert_eq!(visible.len(), 1);
}
