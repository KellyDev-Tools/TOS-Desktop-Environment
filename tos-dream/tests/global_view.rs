use tos_core::*;

#[test]
fn test_global_view_content() {
    let mut state = TosState::new();
    
    // Explicitly set level
    state.current_level = HierarchyLevel::GlobalOverview;
    state.viewports[0].current_level = HierarchyLevel::GlobalOverview;
    
    // Add custom sector to verify dynamic properties
    let description = "Special Research Outpost";
    let icon = "🧪";
    let sector = Sector {
        id: uuid::Uuid::new_v4(),
        name: "RESEARCH-BETA".to_string(),
        hubs: vec![],
        active_hub_index: 0,
        color: "#123456".to_string(),
        host: "local".to_string(),
        connection_type: ConnectionType::Local,
        participants: vec![],
        portal_active: false,
        portal_url: None,
        description: description.to_string(),
        icon: icon.to_string(),
        sector_type_name: "science".to_string(),
    };
    state.sectors.push(sector);
    
    let html = state.render_current_view();
    
    // 4.3 Verify dynamic sector description and icon
    assert!(html.contains(description), "Global view should render dynamic sector description");
    assert!(html.contains(icon), "Global view should render dynamic sector icon");
    
    // 4.1 & 4.2 Verify elements for System Time and Stardate
    assert!(html.contains("tos-sys-time"), "Should contain system time element ID");
    assert!(html.contains("tos-stardate"), "Should contain stardate element ID");
    
    // 4.4 Verify no dev/mock buttons
    if html.contains(">MOCK<") {
        panic!("Global view contains a MOCK button which should be removed");
    }
}
