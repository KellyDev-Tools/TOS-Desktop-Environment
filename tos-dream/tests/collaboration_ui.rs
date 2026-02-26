use tos_core::TosState;
use tos_core::HierarchyLevel;
use tos_core::system::collaboration::{CollaborationRole, Participant};
use tos_core::ui::render::bezel::{render_bezel, BezelState};

#[test]
fn test_avatar_rendering() {
    let mut state = TosState::new_fresh();
    let viewport = state.viewports[0].clone();
    
    // Add a participant with an avatar URL
    let p_id = uuid::Uuid::new_v4();
    state.sectors[0].participants.push(Participant {
        id: p_id,
        name: "Janeway".to_string(),
        color: "#ff0000".to_string(),
        avatar_url: Some("https://example.com/janeway.jpg".to_string()),
        role: CollaborationRole::CoOwner,
        cursor_position: None,
        following_host_id: None,
    });
    
    // Add it to collaboration manager too
    state.collaboration_manager.add_participant(state.sectors[0].participants.last().unwrap().clone(), None);

    let html = render_bezel(&state, &viewport, HierarchyLevel::GlobalOverview, BezelState::Collapsed);
    
    assert!(html.contains("collab-avatar"));
    assert!(html.contains("janeway.jpg"));
    assert!(html.contains("Co-owner"));
}

#[test]
fn test_role_based_restrictions_viewer() {
    let mut state = TosState::new_fresh();
    
    // Change local participant to Viewer
    let local_id = state.local_participant_id;
    if let Some(p) = state.collaboration_manager.participants.get_mut(&local_id) {
        p.role = CollaborationRole::Viewer;
    }
    // Update permission set in session too
    state.collaboration_manager.add_participant(state.collaboration_manager.participants.get(&local_id).unwrap().clone(), None);

    let viewport = state.viewports[0].clone();
    
    // Test Command Hub Bezel (L2)
    let html = render_bezel(&state, &viewport, HierarchyLevel::CommandHub, BezelState::Collapsed);
    
    // Mode switches should be disabled for Viewer
    assert!(html.contains("disabled"), "Viewer should have disabled UI elements");
    assert!(html.contains("toggle-segment active disabled"), "Active segment should be marked disabled");
}

#[test]
fn test_role_based_restrictions_owner() {
    let mut state = TosState::new_fresh();
    
    // Local participant is CoOwner by default
    let viewport = state.viewports[0].clone();
    
    let html = render_bezel(&state, &viewport, HierarchyLevel::GlobalOverview, BezelState::Collapsed);
    
    // Settings and Add Sector should NOT be disabled for Owner
    assert!(!html.contains("disabled"), "Owner should NOT have disabled UI elements");
}
