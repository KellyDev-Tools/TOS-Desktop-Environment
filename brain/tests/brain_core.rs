use tos_common::brain::Brain;
use tos_common::{HierarchyLevel, CommandHubMode};
use std::sync::Arc;

#[tokio::test]
async fn test_brain_initialization() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    let state = brain.state.lock().unwrap();
    
    // Check defaults matches tos-common expectations but within Brain context
    assert_eq!(state.sectors.len(), 1);
    assert_eq!(state.sectors[0].name, "Primary");
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
}

#[tokio::test]
async fn test_ipc_zoom_flow() {
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
    
    // Switch to AI
    let res = brain.ipc.handle_request("set_mode:ai");
    assert_eq!(res, "MODE_SET: Ai");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors[0].hubs[0].mode, CommandHubMode::Ai);
    }
}

#[tokio::test]
async fn test_sector_auto_relabel_and_focus() {
    let brain = Brain::new().expect("Failed to initialize Brain");

    // 1. Initial active sector index should be 0, name should be "Primary"
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.active_sector_index, 0);
        assert_eq!(state.sectors[0].name, "Primary");
    }

    // 2. Create sector with an empty/blank name - should auto-label to "Sector 2" and auto-focus
    let res = brain.ipc.handle_request("sector_create:");
    assert_eq!(res, "SECTOR_CREATED: Sector 2");

    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors.len(), 2);
        assert_eq!(state.sectors[1].name, "Sector 2");
        assert_eq!(state.active_sector_index, 1, "New sector should be auto-focused");
    }

    // 3. Create another sector with blank name - should auto-label to "Sector 3" and focus it
    let res = brain.ipc.handle_request("sector_create:   ");
    assert_eq!(res, "SECTOR_CREATED: Sector 3");

    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors.len(), 3);
        assert_eq!(state.sectors[2].name, "Sector 3");
        assert_eq!(state.active_sector_index, 2);
    }
}

#[tokio::test]
async fn test_sector_limit_ipc() {
    let brain = Brain::new().expect("Failed to initialize Brain");

    // 1. Initial state has 1 sector
    let first_id = {
        let state = brain.state.lock().unwrap();
        state.sectors[0].id.to_string()
    };

    // 2. Try closing the last sector via IPC - should return failure message
    let res = brain.ipc.handle_request(&format!("sector_close:{}", first_id));
    assert!(res.contains("ERROR") || res.contains("FAILED") || res.contains("cannot close the last remaining sector"), "IPC response: {}", res);

    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors.len(), 1, "Sector count should still be 1");
    }
}

