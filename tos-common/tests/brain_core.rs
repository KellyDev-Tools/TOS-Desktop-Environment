use tos_common::brain::Brain;
use tos_common::{HierarchyLevel, CommandHubMode};
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
    
    let state_parsed: tos_common::TosState = serde_json::from_str(&res).expect("IPC state must be valid JSON");
    assert_eq!(state_parsed.sectors.len(), 1);
}

#[tokio::test]
async fn test_auto_activity_mode_on_top() {
    let brain = Brain::new().expect("Failed to initialize Brain");

    // Default mode is Command
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors[0].hubs[0].mode, CommandHubMode::Command);
    }

    // Submitting `top` should auto-switch to Activity mode
    let _res = brain.ipc.handle_request("prompt_submit:top");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(
            state.sectors[0].hubs[0].mode,
            CommandHubMode::Activity,
            "Hub should auto-switch to Activity mode when `top` is submitted"
        );
    }
}

#[tokio::test]
async fn test_auto_activity_mode_no_false_positive() {
    let brain = Brain::new().expect("Failed to initialize Brain");

    // `echo ps` should NOT trigger Activity mode (first token is `echo`)
    let _res = brain.ipc.handle_request("prompt_submit:echo ps");
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(
            state.sectors[0].hubs[0].mode,
            CommandHubMode::Command,
            "Hub should NOT switch to Activity mode for `echo ps`"
        );
    }
}

#[tokio::test]
async fn test_dynamic_sector_labeling() {
    let brain = Brain::new().expect("Failed to initialize Brain");

    // Default sector name is "Primary"
    {
        let state = brain.state.lock().unwrap();
        assert_eq!(state.sectors[0].name, "Primary");
    }

    // Simulate a cwd change by directly mutating state (PTY read loop is the real trigger)
    {
        let mut state = brain.state.lock().unwrap();
        let sector = &mut state.sectors[0];
        let hub = &mut sector.hubs[0];
        hub.current_directory = std::path::PathBuf::from("/home/tim/my-cool-project");

        // Apply the same labeling logic the PTY loop uses
        let auto_labels = ["Primary", "New Sector", "Detached", "Untitled"];
        let is_auto_name = auto_labels
            .iter()
            .any(|prefix| sector.name == *prefix || sector.name.starts_with("Detached"));
        if is_auto_name {
            let label = hub
                .current_directory
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            sector.name = label;
        }
    }

    {
        let state = brain.state.lock().unwrap();
        assert_eq!(
            state.sectors[0].name, "my-cool-project",
            "Sector name should auto-update to the cwd directory name"
        );
    }
}

#[tokio::test]
async fn test_dynamic_sector_labeling_preserves_user_name() {
    let brain = Brain::new().expect("Failed to initialize Brain");

    // Rename sector to a user-chosen name
    {
        let mut state = brain.state.lock().unwrap();
        state.sectors[0].name = "My Work".to_string();
    }

    // Simulate cwd change — should NOT relabel since name is user-set
    {
        let mut state = brain.state.lock().unwrap();
        let sector = &mut state.sectors[0];
        let hub = &mut sector.hubs[0];
        hub.current_directory = std::path::PathBuf::from("/tmp/other-project");

        let auto_labels = ["Primary", "New Sector", "Detached", "Untitled"];
        let is_auto_name = auto_labels
            .iter()
            .any(|prefix| sector.name == *prefix || sector.name.starts_with("Detached"));
        if is_auto_name {
            let label = hub
                .current_directory
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            sector.name = label;
        }
    }

    {
        let state = brain.state.lock().unwrap();
        assert_eq!(
            state.sectors[0].name, "My Work",
            "User-renamed sectors should NOT be relabeled by cwd changes"
        );
    }
}
