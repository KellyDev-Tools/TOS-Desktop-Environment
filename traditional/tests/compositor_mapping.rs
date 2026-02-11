use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::{SurfaceRole, SpatialMapper};

#[test]
fn test_surface_mapping_integration() {
    let mut env = DesktopEnvironment::new(None);
    
    // Create some surfaces across different sectors
    let term = env.surfaces.create_surface("Work Terminal", SurfaceRole::Toplevel, Some(0)); // Sector 0
    let _browser = env.surfaces.create_surface("Music Player", SurfaceRole::Toplevel, Some(1)); // Sector 1
    
    // Level 1: Root
    let layouts = SpatialMapper::get_layout(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        None,
        None,
        4
    );
    assert_eq!(layouts.len(), 4); // Root overview shows sectors

    // Level 2: Zoom into Work Sector (0)
    env.navigator.zoom_in(0);
    let layouts = SpatialMapper::get_layout(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        None,
        None,
        4
    );
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].surface.title, "Work Terminal");
    assert_eq!(layouts[0].grid_x, 0);

    // Level 3: Focus on Terminal
    env.navigator.zoom_in(0);
    let layouts = SpatialMapper::get_layout(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        Some(term),
        None,
        4
    );
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].surface.id, term);
    assert_eq!(layouts[0].width, 3); // Spans full grid
}

#[test]
fn test_picker_mapping() {
    let mut env = DesktopEnvironment::new(None);
    
    let w1 = env.surfaces.create_surface("Term Window 1", SurfaceRole::Toplevel, Some(0));
    let _w2 = env.surfaces.create_surface("Term Window 2", SurfaceRole::Toplevel, Some(0));
    
    env.navigator.zoom_in(0);
    env.navigator.zoom_in(0); 
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Focus);
    
    env.navigator.active_app_index = Some(0);
    env.intelligent_zoom_out();
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3aPicker);
    
    let layouts = SpatialMapper::get_layout(
        &env.surfaces, 
        env.navigator.current_level, 
        env.navigator.active_sector_index, 
        Some(w1),
        None,
        4
    );
    assert_eq!(layouts.len(), 2);
}

#[test]
fn test_split_view_layout() {
    let mut env = DesktopEnvironment::new(None);
    let s1 = env.surfaces.create_surface("S1", SurfaceRole::Toplevel, Some(0));
    let s2 = env.surfaces.create_surface("S2", SurfaceRole::Toplevel, Some(0));

    env.navigator.zoom_in(0); // Sector
    env.navigator.zoom_in(0); // Focus s1
    
    env.navigator.split_view(s2);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Split);

    let layouts = SpatialMapper::get_layout(
        &env.surfaces,
        env.navigator.current_level,
        env.navigator.active_sector_index,
        Some(s1),
        Some(s2),
        4
    );

    assert_eq!(layouts.len(), 2);
    // S1 should be span 2
    assert_eq!(layouts[0].width, 2);
    // S2 should be span 1
    assert_eq!(layouts[1].width, 1);
}

#[test]
fn test_adaptive_sector_layout() {
    let mut env = DesktopEnvironment::new(None);
    
    // Case 1: Single app
    env.surfaces.create_surface("Only One", SurfaceRole::Toplevel, Some(0));
    env.navigator.zoom_in(0);
    let layouts = SpatialMapper::get_layout(&env.surfaces, ZoomLevel::Level2Sector, Some(0), None, None, 4);
    assert_eq!(layouts[0].width, 3); // Spans full

    // Case 2: Two apps
    env.surfaces.create_surface("Secondary", SurfaceRole::Toplevel, Some(0));
    let layouts = SpatialMapper::get_layout(&env.surfaces, ZoomLevel::Level2Sector, Some(0), None, None, 4);
    assert_eq!(layouts[0].width, 2);
    assert_eq!(layouts[1].width, 1);
}

#[test]
fn test_swap_split() {
    let mut env = DesktopEnvironment::new(None);
    let s1 = env.surfaces.create_surface("S1", SurfaceRole::Toplevel, Some(0));
    let s2 = env.surfaces.create_surface("S2", SurfaceRole::Toplevel, Some(0));

    env.navigator.zoom_in(0); // Sector
    env.navigator.zoom_in(0); // Focus s1
    env.navigator.split_view(s2);

    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Split);
    assert_eq!(env.navigator.secondary_app_id, Some(s2));
    
    // Swap
    let success = env.swap_split();
    assert!(success);
    
    // Primary should now be S2
    let primary_id = if let (Some(sector), Some(app)) = (env.navigator.active_sector_index, env.navigator.active_app_index) {
        env.surfaces.get_surfaces_in_sector(sector).get(app).map(|s| s.id)
    } else { None };
    
    assert_eq!(primary_id, Some(s2));
    assert_eq!(env.navigator.secondary_app_id, Some(s1));
}
