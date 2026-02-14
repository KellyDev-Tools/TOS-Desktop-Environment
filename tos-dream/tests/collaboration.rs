use tos_core::*;

#[test]
fn test_sector_management() {
    let mut state = TosState::new();
    
    // Initial state: 2 sectors (Alpha, Science)
    assert_eq!(state.sectors.len(), 2);
    
    // Select second sector
    state.select_sector(1);
    assert_eq!(state.viewports[0].sector_index, 1);
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    
    // Add a new sector
    let new_sector = Sector {
        id: uuid::Uuid::new_v4(),
        name: "Engineering".to_string(),
        color: "#ffcc00".to_string(),
        hubs: vec![CommandHub {
            id: uuid::Uuid::new_v4(),
            mode: CommandHubMode::Command,
            prompt: String::new(),
            applications: Vec::new(),
            active_app_index: None,
            terminal_output: Vec::new(),
            confirmation_required: None,
        }],
        active_hub_index: 0,
        host: "LOCAL".to_string(),
        is_remote: false,
        participants: Vec::new(),
    };
    state.add_sector(new_sector);
    assert_eq!(state.sectors.len(), 3);
    assert_eq!(state.sectors[2].name, "Engineering");
}

#[test]
fn test_collaboration_participants() {
    let mut state = TosState::new();
    
    // Add participant to first sector
    state.add_participant(0, "Geordi".to_string(), "#ffcc00".to_string(), "Engineer".to_string());
    
    assert_eq!(state.sectors[0].participants.len(), 2); // Host + Geordi
    assert_eq!(state.sectors[0].participants[1].name, "Geordi");
    
    // Verify rendering contains participant
    state.select_sector(0);
    let html = state.render_current_view();
    assert!(html.contains("Geordi"));
}

#[test]
fn test_app_focus_by_id() {
    let mut state = TosState::new();
    state.select_sector(1); // Science Labs
    
    let app_id = state.sectors[1].hubs[0].applications[1].id; // Stellar Cartography
    state.focus_app_by_id(app_id);
    
    assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    assert_eq!(state.sectors[1].hubs[0].active_app_index, Some(1));
    
    let html = state.render_current_view();
    println!("DEBUG HTML: {}", html);
    assert!(html.contains("STELLAR CARTOGRAPHY"));
}
