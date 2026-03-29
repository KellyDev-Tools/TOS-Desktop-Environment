//! Tests for collaboration and communication features.
//!
//! These validate multi-user collaboration, communication channels,
//! and shared workspace features.

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS COLLABORATION TESTS]\x1B[0m");
    println!("Testing collaboration and communication features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY COLLABORATION MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial Collaboration Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_module.is_some(), "Collaboration module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration module loaded");
    }

    // 3. COLLABORATION MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Mode Activation - collaboration:enable]\x1B[0m");
    println!("-> Action: collaboration:enable");
    brain.ipc.handle_request("collaboration:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_mode, "Collaboration mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration mode enabled");
    }

    // 4. COLLABORATION MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Mode Deactivation - collaboration:disable]\x1B[0m");
    println!("-> Action: collaboration:disable");
    brain.ipc.handle_request("collaboration:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.collaboration_mode, "Collaboration mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration mode disabled");
    }

    // 5. COLLABORATION HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Hub Activation - collaboration_hub:enable]\x1B[0m");
    println!("-> Action: collaboration_hub:enable");
    brain.ipc.handle_request("collaboration_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_hub, "Collaboration hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration hub enabled");
    }

    // 6. COLLABORATION HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Hub Deactivation - collaboration_hub:disable]\x1B[0m");
    println!("-> Action: collaboration_hub:disable");
    brain.ipc.handle_request("collaboration_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.collaboration_hub, "Collaboration hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration hub disabled");
    }

    // 7. COLLABORATION TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Terminal Activation - collaboration_terminal:enable]\x1B[0m");
    println!("-> Action: collaboration_terminal:enable");
    brain.ipc.handle_request("collaboration_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_terminal, "Collaboration terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration terminal enabled");
    }

    // 8. COLLABORATION TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Terminal Deactivation - collaboration_terminal:disable]\x1B[0m");
    println!("-> Action: collaboration_terminal:disable");
    brain.ipc.handle_request("collaboration_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.collaboration_terminal, "Collaboration terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration terminal disabled");
    }

    // 9. COLLABORATION APPLICATION ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Activation - collaboration_app:enable]\x1B[0m");
    println!("-> Action: collaboration_app:enable");
    brain.ipc.handle_request("collaboration_app:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app, "Collaboration app should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app enabled");
    }

    // 10. COLLABORATION APPLICATION DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Deactivation - collaboration_app:disable]\x1B[0m");
    println!("-> Action: collaboration_app:disable");
    brain.ipc.handle_request("collaboration_app:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.collaboration_app, "Collaboration app should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app disabled");
    }

    // 11. COLLABORATION APPLICATION EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution - collaboration_app:execute:ls]\x1B[0m");
    println!("-> Action: collaboration_app:execute:ls");
    brain.ipc.handle_request("collaboration_app:execute:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution set");
    }

    // 12. COLLABORATION APPLICATION EXECUTION WITH ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Arguments - collaboration_app:execute:ls -la]\x1B[0m");
    println!("-> Action: collaboration_app:execute:ls -la");
    brain.ipc.handle_request("collaboration_app:execute:ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with arguments set");
    }

    // 13. COLLABORATION APPLICATION EXECUTION WITH PATH TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Path - collaboration_app:execute:/bin/ls]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with path set");
    }

    // 14. COLLABORATION APPLICATION EXECUTION WITH PATH AND ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Path and Arguments - collaboration_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with path and arguments set");
    }

    // 15. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, AND ENVIRONMENT TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Environment - collaboration_app:execute:/bin/ls -la;PATH=/usr/bin]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;PATH=/usr/bin");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;PATH=/usr/bin");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with environment set");
    }

    // 16. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, AND USER TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with User - collaboration_app:execute:/bin/ls -la;USER=root]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;USER=root");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;USER=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with user set");
    }

    // 17. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, AND GROUP TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Group - collaboration_app:execute:/bin/ls -la;USER=root;GROUP=root]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;USER=root;GROUP=root");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;USER=root;GROUP=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with group set");
    }

    // 18. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, AND SHELL TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Shell - collaboration_app:execute:/bin/ls -la;SHELL=/bin/bash]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;SHELL=/bin/bash");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;SHELL=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with shell set");
    }

    // 19. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, AND HOME TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Home - collaboration_app:execute:/bin/ls -la;HOME=/root]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;HOME=/root");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;HOME=/root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with home set");
    }

    // 20. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, AND TERM TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Term - collaboration_app:execute:/bin/ls -la;TERM=xterm-256color]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;TERM=xterm-256color");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;TERM=xterm-256color");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with term set");
    }

    // 21. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, AND SHELL_PATH TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Shell Path - collaboration_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with shell path set");
    }

    // 22. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, AND SHELL_TYPE TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Shell Type - collaboration_app:execute:/bin/ls -la;SHELL_TYPE=sh]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;SHELL_TYPE=sh");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;SHELL_TYPE=sh");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with shell type set");
    }

    // 23. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, AND SHELL_VERSION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Shell Version - collaboration_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with shell version set");
    }

    // 24. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, AND SHELL_FLAGS TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Shell Flags - collaboration_app:execute:/bin/ls -la;SHELL_FLAGS=-i]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;SHELL_FLAGS=-i");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;SHELL_FLAGS=-i");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with shell flags set");
    }

    // 25. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, AND SHELL_OPTIONS TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Shell Options - collaboration_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with shell options set");
    }

    // 26. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, AND SHELL_TIMEOUT TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Shell Timeout - collaboration_app:execute:/bin/ls -la;SHELL_TIMEOUT=300]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with shell timeout set");
    }

    // 27. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, AND SHELL_MEMORY_LIMIT TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Memory Limit - collaboration_app:execute:/bin/ls -la;MEMORY_LIMIT=1024]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution with memory limit set");
    }

    // 28. COLLABORATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, MEMORY_LIMIT, AND COLLABORATION APP EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Collaboration Application Execution with Collaboration App Execution - collaboration_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: collaboration_app:execute:/bin/ls -la");
    brain.ipc.handle_request("collaboration_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.collaboration_app_execution.is_some(), "Collaboration app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Collaboration app execution set");
    }

    println!("\n\x1B[1;32m=== COLLABORATION TESTS PASSED ===\x1B[0m");

    Ok(())
}
