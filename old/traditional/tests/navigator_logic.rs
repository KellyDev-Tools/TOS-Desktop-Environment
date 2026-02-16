use tos_comp::navigation::zoom::{SpatialNavigator, ZoomLevel};

#[test]
fn test_navigator_reset_on_zoom_out() {
    let mut nav = SpatialNavigator::new();
    
    // Zoom in deep
    nav.zoom_in(0); // Sector
    nav.zoom_in(0); // Focus
    nav.split_view(10); // Split
    
    assert_eq!(nav.current_level, ZoomLevel::Level3Split);
    assert_eq!(nav.secondary_app_id, Some(10));
    
    // Zoom out from split
    nav.zoom_out(false);
    assert_eq!(nav.current_level, ZoomLevel::Level3Focus);
    assert!(nav.secondary_app_id.is_none()); // Should be reset
}

#[test]
fn test_navigator_picker_logic() {
    let mut nav = SpatialNavigator::new();
    
    nav.zoom_in(0); // Sector
    nav.zoom_in(0); // Focus
    
    // Zoom out with multiple windows
    nav.zoom_out(true);
    assert_eq!(nav.current_level, ZoomLevel::Level3aPicker);
    
    // Selecting from picker
    nav.zoom_in(1); // Select 2nd window
    assert_eq!(nav.current_level, ZoomLevel::Level3Focus);
    assert_eq!(nav.active_window_index, Some(1));
}

#[test]
fn test_zoom_out_from_detail() {
    let mut nav = SpatialNavigator::new();
    nav.zoom_in(0); // Sector
    nav.zoom_in(0); // Focus
    nav.zoom_in(0); // Detail
    
    assert_eq!(nav.current_level, ZoomLevel::Level4Detail);
    
    nav.zoom_out(false);
    assert_eq!(nav.current_level, ZoomLevel::Level3Focus);
}

#[test]
fn test_navigator_jump_integrity() {
    let mut nav = SpatialNavigator::new();
    
    // Start at root
    assert_eq!(nav.current_level, ZoomLevel::Level1Root);
    
    // Zoom into sector 2
    nav.zoom_in(2);
    assert_eq!(nav.active_sector_index, Some(2));
    
    // Zoom back out
    nav.zoom_out(false);
    assert!(nav.active_sector_index.is_none());
}
