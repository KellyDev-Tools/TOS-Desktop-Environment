// Tests for user_app features.

use std::collections::HashMap;
use tokio::time::sleep;
use tos_brain::Brain;
use tos_brain::common::CommandHubMode;
use tos_brain::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS USER_APP TESTS]\x1B[0m");
    println!("Testing user_app features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY USER_APP MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial User_App Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_module.is_some(), "User_app module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_app module loaded");
    }

    // 3. USER_APP MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_App Mode Activation - user_app:enable]\x1B[0m");
    println!("-> Action: user_app:enable");
    brain.ipc.handle_request("user_app:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_mode, "User_app mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_app mode enabled");
    }

    // 4. USER_APP MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_App Mode Deactivation - user_app:disable]\x1B[0m");
    println!("-> Action: user_app:disable");
    brain.ipc.handle_request("user_app:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_app_mode, "User_app mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_app mode disabled");
    }

    // 5. USER_APP HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_App Hub Activation - user_app_hub:enable]\x1B[0m");
    println!("-> Action: user_app_hub:enable");
    brain.ipc.handle_request("user_app_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_hub, "User_app hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_app hub enabled");
    }

    // 6. USER_APP HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_App Hub Deactivation - user_app_hub:disable]\x1B[0m");
    println!("-> Action: user_app_hub:disable");
    brain.ipc.handle_request("user_app_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_app_hub, "User_app hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_app hub disabled");
    }

    // 7. USER_APP TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_App Terminal Activation - user_app_terminal:enable]\x1B[0m");
    println!("-> Action: user_app_terminal:enable");
    brain.ipc.handle_request("user_app_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_terminal, "User_app terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_app terminal enabled");
    }

    // 8. USER_APP TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_App Terminal Deactivation - user_app_terminal:disable]\x1B[0m");
    println!("-> Action: user_app_terminal:disable");
    brain.ipc.handle_request("user_app_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_app_terminal, "User_app terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_app terminal disabled");
    }

    println!("\n\x1B[1;32m=== USER_APP TESTS PASSED ===\x1B[0m");

    Ok(())
}
