use tos_core::*;
use tos_core::system::collaboration::*;

#[test]
fn test_sector_management() {
    let mut state = TosState::new();
    
    // Initial state: 3 sectors (Alpha, Science, Observation Hub)
    assert_eq!(state.sectors.len(), 3);
    
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
            current_directory: std::path::PathBuf::from("/"),
            show_hidden_files: false,
            selected_files: std::collections::HashSet::new(),
            context_menu: None,
        }],
        active_hub_index: 0,
        host: "LOCAL".to_string(),
        connection_type: ConnectionType::Local,
        participants: Vec::new(),
        portal_active: false,
        portal_url: None,
        description: "Main engineering and power control.".to_string(),
        icon: "⚙️".to_string(),
    };
    state.add_sector(new_sector);
    assert_eq!(state.sectors.len(), 4);
    assert_eq!(state.sectors[3].name, "Engineering");
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
    assert!(html.contains("DATAFEED"));
}

#[test]
fn test_invitation_system() {
    let mut state = TosState::new();
    let sector_id = state.sectors[0].id;
    
    // Create invitation
    let token = state.collaboration_manager.create_invitation(sector_id, CollaborationRole::Operator);
    assert!(!token.is_empty());
    
    // Redeem invitation
    let result = state.collaboration_manager.redeem_invitation(&token);
    assert!(result.is_some());
    let (redeemed_sector, redeemed_role) = result.unwrap();
    assert_eq!(redeemed_sector, sector_id);
    assert_eq!(redeemed_role, CollaborationRole::Operator);
    
    // Token should now be used
    let result_second = state.collaboration_manager.redeem_invitation(&token);
    assert!(result_second.is_none());
}

#[test]
fn test_rbac_permissions() {
    let co_owner = CollaborationRole::CoOwner;
    let operator = CollaborationRole::Operator;
    let viewer = CollaborationRole::Viewer;
    
    let co_owner_perms = PermissionSet::for_role(co_owner);
    let operator_perms = PermissionSet::for_role(operator);
    let viewer_perms = PermissionSet::for_role(viewer);
    
    assert!(co_owner_perms.allow_sector_reset);
    assert!(co_owner_perms.allow_shell_input);
    
    assert!(!operator_perms.allow_sector_reset);
    assert!(operator_perms.allow_shell_input);
    
    assert!(!viewer_perms.allow_shell_input);
    assert!(!viewer_perms.allow_app_launch);
}
