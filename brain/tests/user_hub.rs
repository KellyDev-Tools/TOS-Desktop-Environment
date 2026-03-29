// Tests for user_hub features.

use std::collections::HashMap;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS USER_HUB TESTS]\x1B[0m");
    println!("Testing user_hub features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY USER_HUB MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial User_Hub Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_hub_module.is_some(), "User_hub module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_hub module loaded");
    }

    // 3. USER_HUB MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Hub Mode Activation - user_hub:enable]\x1B[0m");
    println!("-> Action: user_hub:enable");
    brain.ipc.handle_request("user_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_hub_mode, "User_hub mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_hub mode enabled");
    }

    // 4. USER_HUB MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Hub Mode Deactivation - user_hub:disable]\x1B[0m");
    println!("-> Action: user_hub:disable");
    brain.ipc.handle_request("user_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_hub_mode, "User_hub mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_hub mode disabled");
    }

    // 5. USER_HUB HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Hub Hub Activation - user_hub_hub:enable]\x1B[0m");
    println!("-> Action: user_hub_hub:enable");
    brain.ipc.handle_request("user_hub_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_hub_hub, "User_hub hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_hub hub enabled");
    }

    // 6. USER_HUB HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Hub Hub Deactivation - user_hub_hub:disable]\x1B[0m");
    println!("-> Action: user_hub_hub:disable");
    brain.ipc.handle_request("user_hub_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_hub_hub, "User_hub hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_hub hub disabled");
    }

    // 7. USER_HUB TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Hub Terminal Activation - user_hub_terminal:enable]\x1B[0m");
    println!("-> Action: user_hub_terminal:enable");
    brain.ipc.handle_request("user_hub_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_hub_terminal, "User_hub terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_hub terminal enabled");
    }

    // 8. USER_HUB TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Hub Terminal Deactivation - user_hub_terminal:disable]\x1B[0m");
    println!("-> Action: user_hub_terminal:disable");
    brain.ipc.handle_request("user_hub_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_hub_terminal, "User_hub terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_hub terminal disabled");
    }

    println!("\n\x1B[1;32m=== USER_HUB TESTS PASSED ===\x1B[0m");

    Ok(())
}
