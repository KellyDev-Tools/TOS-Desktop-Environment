use crate::brain::Brain;
use crate::common::{HierarchyLevel, CommandHubMode};
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
async fn test_brain_initialization() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    let state = brain.state.lock().unwrap();
    assert_eq!(state.sectors.len(), 1);
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
}

#[tokio::test]
async fn test_ipc_zoom_flow() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // Zoom In 
    let res = brain.ipc.handle_request("zoom_in");
    assert_eq!(res, "ZOOMED_IN");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    }
}

#[tokio::test]
async fn test_sector_management() {
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

#[tokio::test]
async fn test_mode_switching() {
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
}

#[tokio::test]
async fn test_get_state_json() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    let res = brain.ipc.handle_request("get_state");
    assert!(res.starts_with("{")); // Should be JSON
    
    let state_parsed: crate::common::TosState = serde_json::from_str(&res).expect("IPC state must be valid JSON");
    assert_eq!(state_parsed.sectors.len(), 1);
}
