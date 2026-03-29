// Tests for user_terminal features.

use std::collections::HashMap;
use tokio::time::sleep;
use tos_brain::Brain;
use tos_brain::common::CommandHubMode;
use tos_brain::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS USER_TERMINAL TESTS]\x1B[0m");
    println!("Testing user_terminal features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY USER_TERMINAL MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial User_Terminal Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_terminal_module.is_some(), "User_terminal module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_terminal module loaded");
    }

    // 3. USER_TERMINAL MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Terminal Mode Activation - user_terminal:enable]\x1B[0m");
    println!("-> Action: user_terminal:enable");
    brain.ipc.handle_request("user_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_terminal_mode, "User_terminal mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_terminal mode enabled");
    }

    // 4. USER_TERMINAL MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Terminal Mode Deactivation - user_terminal:disable]\x1B[0m");
    println!("-> Action: user_terminal:disable");
    brain.ipc.handle_request("user_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_terminal_mode, "User_terminal mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_terminal mode disabled");
    }

    // 5. USER_TERMINAL HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Terminal Hub Activation - user_terminal_hub:enable]\x1B[0m");
    println!("-> Action: user_terminal_hub:enable");
    brain.ipc.handle_request("user_terminal_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_terminal_hub, "User_terminal hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_terminal hub enabled");
    }

    // 6. USER_TERMINAL HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Terminal Hub Deactivation - user_terminal_hub:disable]\x1B[0m");
    println!("-> Action: user_terminal_hub:disable");
    brain.ipc.handle_request("user_terminal_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_terminal_hub, "User_terminal hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_terminal hub disabled");
    }

    // 7. USER_TERMINAL TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Terminal Terminal Activation - user_terminal_terminal:enable]\x1B[0m");
    println!("-> Action: user_terminal_terminal:enable");
    brain.ipc.handle_request("user_terminal_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_terminal_terminal, "User_terminal terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_terminal terminal enabled");
    }

    // 8. USER_TERMINAL TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User_Terminal Terminal Deactivation - user_terminal_terminal:disable]\x1B[0m");
    println!("-> Action: user_terminal_terminal:disable");
    brain.ipc.handle_request("user_terminal_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_terminal_terminal, "User_terminal terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User_terminal terminal disabled");
    }

    println!("\n\x1B[1;32m=== USER_TERMINAL TESTS PASSED ===\x1B[0m");

    Ok(())
}
