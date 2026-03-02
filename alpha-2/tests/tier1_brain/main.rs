use tos_alpha2::common::{TosState, HierarchyLevel, CommandHubMode};
use tos_alpha2::brain::hierarchy::HierarchyManager;
use tos_alpha2::brain::sector::SectorManager;

#[test]
fn test_hierarchy_transitions() {
    let mut state = TosState::default();
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);

    // Zoom in chain
    assert!(HierarchyManager::zoom_in(&mut state));
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);

    assert!(HierarchyManager::zoom_in(&mut state));
    assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);

    assert!(HierarchyManager::zoom_in(&mut state));
    assert_eq!(state.current_level, HierarchyLevel::DetailView);

    // Level 5 (BufferView) should fail without setting
    assert!(!HierarchyManager::zoom_in(&mut state));
    assert_eq!(state.current_level, HierarchyLevel::DetailView);

    // Enable deep inspection
    state.settings.global.insert("deep_inspection".to_string(), "true".to_string());
    assert!(HierarchyManager::zoom_in(&mut state));
    assert_eq!(state.current_level, HierarchyLevel::BufferView);

    // Zoom out chain
    assert!(HierarchyManager::zoom_out(&mut state));
    assert_eq!(state.current_level, HierarchyLevel::DetailView);
}

#[test]
fn test_sector_lifecycle() {
    let mut state = TosState::default();
    let initial_count = state.sectors.len();

    // Create
    let new_id = SectorManager::create_sector(&mut state, "Test Sector".to_string());
    assert_eq!(state.sectors.len(), initial_count + 1);
    assert_eq!(state.sectors.last().unwrap().name, "Test Sector");

    // Clone
    let clone_id = SectorManager::clone_sector(&mut state, new_id).expect("Clone failed");
    assert_eq!(state.sectors.len(), initial_count + 2);
    assert!(state.sectors.last().unwrap().name.contains("Clone"));
    assert_ne!(state.sectors.last().unwrap().id, new_id);

    // Freeze
    SectorManager::toggle_freeze(&mut state, clone_id);
    assert!(state.sectors.iter().find(|s| s.id == clone_id).unwrap().frozen);
    SectorManager::toggle_freeze(&mut state, clone_id);
    assert!(!state.sectors.iter().find(|s| s.id == clone_id).unwrap().frozen);

    // Close
    assert!(SectorManager::close_sector(&mut state, new_id));
    assert_eq!(state.sectors.len(), initial_count + 1);
}

#[test]
fn test_directory_listing_fallback() {
    let mut state = TosState::default();
    // Force Directory mode
    state.sectors[0].hubs[0].mode = CommandHubMode::Directory;
    
    // Refresh
    SectorManager::refresh_directory_listing(&mut state);
    
    let hub = &state.sectors[0].hubs[0];
    assert!(hub.shell_listing.is_some());
    let listing = hub.shell_listing.as_ref().unwrap();
    // Should have some entries from the current dir (alpha-2 root)
    assert!(!listing.entries.is_empty());
}
