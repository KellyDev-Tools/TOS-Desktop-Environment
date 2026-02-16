//! UI Visual Logging and Coverage Verification
//! This test file captures the state of the UI at critical points
//! and saves snapshots to the log directory.

use tos_core::{TosState, HierarchyLevel, CommandHubMode};
use tos_core::containers::sandbox::{SandboxManager, SandboxLevel};
use tos_core::containers::ContainerManager;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use chrono::Local;

fn get_log_dir() -> String {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let path = format!("log/{}/ui_snapshots", date);
    fs::create_dir_all(&path).unwrap();
    path
}

fn save_snapshot(state: &TosState, test_name: &str, step: &str) {
    let dir = get_log_dir();
    
    // Save HTML
    let html = state.render_current_view();
    let html_filename = format!("{}/{}_{}.html", dir, test_name, step);
    fs::write(&html_filename, html).unwrap();
    
    // Save SVG
    let svg = tos_core::ui::render::svg_engine::render_state_to_svg(state);
    let svg_filename = format!("{}/{}_{}.svg", dir, test_name, step);
    fs::write(&svg_filename, svg).unwrap();
    
    println!("  [UI LOG] Saved snapshots: {} and {}", html_filename, svg_filename);
}

#[tokio::test]
async fn test_ui_flow_with_logging() {
    println!("Starting UI Flow Visual Log...");
    let mut state = TosState::new();
    
    // 1. Global Overview
    println!("Step 1: Global Overview");
    save_snapshot(&state, "main_flow", "01_global_overview");
    
    // 2. Command Hub - Directory Mode
    println!("Step 2: Command Hub - Directory Mode");
    state.active_viewport_index = 0;
    state.zoom_in();
    state.toggle_mode(CommandHubMode::Directory);
    save_snapshot(&state, "main_flow", "02_command_hub_directory");
    
    // 3. Command Hub - Activity Mode
    println!("Step 3: Command Hub - Activity Mode");
    state.toggle_mode(CommandHubMode::Activity);
    save_snapshot(&state, "main_flow", "03_command_hub_activity");
    
    // 4. Application Focus
    println!("Step 4: Application Focus");
    state.zoom_in();
    save_snapshot(&state, "main_flow", "04_app_focus");
    
    // 5. Tactical Bezel Expanded
    println!("Step 5: Tactical Bezel Expanded");
    state.viewports[0].bezel_expanded = true;
    save_snapshot(&state, "main_flow", "05_bezel_expanded");
    
    // 6. Mini-Map Active
    println!("Step 6: Mini-Map Active");
    // Mini-map is usually a separate render call, but we can verify state
    save_snapshot(&state, "main_flow", "06_minimap_state");
}

#[tokio::test]
async fn test_sandbox_coverage() {
    println!("Verifying SandboxManager coverage...");
    
    // We need a container manager first
    let container_manager = Arc::new(ContainerManager::new(tos_core::containers::ContainerBackend::Mock).await.unwrap());
    let sandbox_manager = SandboxManager::new(container_manager);
    
    // Test creation of different levels
    let levels = vec![
        SandboxLevel::None,
        SandboxLevel::Standard,
        SandboxLevel::Restricted,
        SandboxLevel::Paranoid,
    ];
    
    for level in levels {
        let id = format!("test-sb-{:?}", level).to_lowercase();
        println!("  Testing sandbox level: {:?}", level);
        let info = sandbox_manager.create_sandbox(&id, level).await.unwrap();
        assert_eq!(info.level, level);
        assert!(info.active);
    }
    
    // Test listing
    let active = sandbox_manager.list_sandboxes();
    assert_eq!(active.len(), 4);
    
    // Test termination
    sandbox_manager.terminate_sandbox("test-sb-paranoid").await.unwrap();
    let active = sandbox_manager.list_sandboxes();
    assert_eq!(active.len(), 3);
    
    println!("SandboxManager coverage verified.");
}

#[tokio::test]
async fn test_split_view_visual_log() {
    println!("Starting Split View Visual Log...");
    let mut state = TosState::new();
    
    // Create splits
    state.zoom_in(); // Hub
    
    // Manually add viewports for testing split layout
    let vp2 = tos_core::Viewport {
        id: uuid::Uuid::new_v4(),
        sector_index: 1,
        hub_index: 0,
        current_level: HierarchyLevel::CommandHub,
        active_app_index: None,
        bezel_expanded: false,
    };
    state.viewports.push(vp2);
    state.current_level = HierarchyLevel::SplitView;
    
    println!("Step 1: Horizontal Split");
    save_snapshot(&state, "split_view", "01_horizontal_split");
    
    println!("Split View visual log complete.");
}
