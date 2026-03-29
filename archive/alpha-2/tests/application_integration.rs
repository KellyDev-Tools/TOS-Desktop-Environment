use tos_common::common::{TosState, HierarchyLevel, ApplicationModel, DecorationPolicy, ZoomBehavior};
use tos_common::brain::sector::SectorManager;
// unused

#[test]
fn test_application_lifecycle_round_trip() {
    let mut state = TosState::default();
    
    // 1. Verify initial level (Global Overview / Level 1)
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    
    // 2. Define an Application Model (§8.2)
    let browser_model = ApplicationModel {
        id: "tactical-browser".to_string(),
        name: "Tactical Browser".to_string(),
        version: "1.0.0".to_string(),
        icon: "globe.svg".to_string(),
        bezel_actions: vec![],
        decoration_policy: DecorationPolicy::Suppress,
        zoom_behavior: ZoomBehavior::Internal,
        searchable_content: true,
    };
    
    // 3. Zoom into Sector 0 (Command Hub / Level 2)
    state.active_sector_index = 0;
    state.current_level = HierarchyLevel::CommandHub; 
    let sector_id = state.sectors[0].id;

    // 4. Launch Application (Application Focus / Level 3)
    let app_id = SectorManager::launch_app(&mut state, sector_id, browser_model.clone());
    
    // Verify State Change (§1.1)
    assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    {
        let sector = &state.sectors[0];
        assert_eq!(sector.active_apps.len(), 1);
        assert_eq!(sector.active_apps[0].id, app_id);
        assert_eq!(sector.active_apps[0].model_id, "tactical-browser");
    }

    // 5. Close Application (Fallback to Level 2)
    SectorManager::close_app(&mut state, sector_id, app_id);
    
    // Verify Level Reversion (§1.1)
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    assert_eq!(state.sectors[0].active_apps.len(), 0);
}
