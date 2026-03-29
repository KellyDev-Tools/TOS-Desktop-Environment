use tos_common::brain::Brain;
use tos_common::common::{HierarchyLevel, CommandHubMode};
use std::sync::Arc;

#[test]
fn test_brain_initialization() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    let state = brain.state.lock().unwrap();
    
    // Check defaults matches tos-common expectations but within Brain context
    assert_eq!(state.sectors.len(), 1);
    assert_eq!(state.sectors[0].name, "Primary");
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
}

#[test]
fn test_ipc_zoom_flow() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // Level 1: GlobalOverview
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    }
    
    // Zoom In -> Level 2: CommandHub
    let res = brain.ipc.handle_request("zoom_in");
    assert_eq!(res, "ZOOMED_IN");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    }
    
    // Zoom In -> Level 3: ApplicationFocus
    let res = brain.ipc.handle_request("zoom_in");
    assert_eq!(res, "ZOOMED_IN");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    }
    
    // Zoom Out -> Level 2: CommandHub
    let res = brain.ipc.handle_request("zoom_out");
    assert_eq!(res, "ZOOMED_OUT");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    }
}

#[test]
fn test_sector_management() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // Initial state
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors.len(), 1);
    }
    
    // Create new sector
    let res = brain.ipc.handle_request("sector_create:Research");
    assert_eq!(res, "SECTOR_CREATED: Research");
    
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors.len(), 2);
        assert_eq!(state.sectors[1].name, "Research");
    }
    
    // Close sector
    let sector_id = {
        let state = brain.state.lock().unwrap();
        state.sectors[1].id.to_string()
    };
    
    let res = brain.ipc.handle_request(&format!("sector_close:{}", sector_id));
    assert!(res.starts_with("SECTOR_CLOSED:"));
    
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors.len(), 1);
    }
}

#[test]
fn test_mode_switching() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // Default mode is Command
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors[0].hubs[0].mode, CommandHubMode::Command);
    }
    
    // Switch to Directory
    let res = brain.ipc.handle_request("set_mode:directory");
    assert_eq!(res, "MODE_SET: Directory");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors[0].hubs[0].mode, CommandHubMode::Directory);
    }
    
    // Switch to AI
    let res = brain.ipc.handle_request("set_mode:ai");
    assert_eq!(res, "MODE_SET: Ai");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors[0].hubs[0].mode, CommandHubMode::Ai);
    }
}
