use tos_common::brain::Brain;
use tos_common::state::HierarchyLevel;

#[tokio::test]
async fn test_switch_scanning_flow() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // 1. Initial state: scanning disabled
    {
        let state = brain.state.lock().unwrap();
        assert!(!state.accessibility.scanning_enabled);
    }
    
    // 2. Toggle scanning ON
    let res = brain.ipc.handle_request("access_scan_toggle");
    assert_eq!(res, "OK");
    
    {
        let state = brain.state.lock().unwrap();
        assert!(state.accessibility.scanning_enabled);
        assert!(!state.accessibility.active_scan_path.is_empty());
        assert_eq!(state.accessibility.current_scan_index, 0);
        // Default first element in path is level:global (GlobalOverview)
        assert_eq!(state.accessibility.active_scan_path[0], "level:global");
    }
    
    // 3. Advance scan
    let res = brain.ipc.handle_request("access_scan_advance");
    assert_eq!(res, "OK");
    
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.accessibility.current_scan_index, 1);
        assert_eq!(state.accessibility.active_scan_path[1], "level:hubs");
    }
    
    // 4. Select "level:hubs" (CommandHub)
    let res = brain.ipc.handle_request("access_scan_select");
    assert_eq!(res, "LEVEL_SET: CommandHub");
    
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    }
}

#[tokio::test]
async fn test_sector_scanning() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // Create a second sector
    brain.ipc.handle_request("sector_create:Research");
    
    // Zoom back out to Global Overview to enable sector scanning
    brain.ipc.handle_request("set_mode:global");
    
    // Toggle scanning ON
    brain.ipc.handle_request("access_scan_toggle");
    
    {
        let state = brain.state.lock().unwrap();
        // Path should include level navigation + both sectors
        assert!(state.accessibility.active_scan_path.contains(&"sector:0".to_string()));
        assert!(state.accessibility.active_scan_path.contains(&"sector:1".to_string()));
    }
    
    // Find index of sector:1 in scan path
    let target_idx = {
        let state = brain.state.lock().unwrap();
        state.accessibility.active_scan_path.iter().position(|r| r == "sector:1").unwrap()
    };
    
    // Advance scan until we reach sector:1
    for _ in 0..target_idx {
        brain.ipc.handle_request("access_scan_advance");
    }
    
    // Select sector:1
    let res = brain.ipc.handle_request("access_scan_select");
    assert_eq!(res, "ACTIVE_SECTOR_SET: 1");
    
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.active_sector_index, 1);
    }
}
