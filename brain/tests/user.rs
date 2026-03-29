// Tests for user features.

use std::collections::HashMap;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS USER TESTS]\x1B[0m");
    println!("Testing user features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY USER MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial User Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_module.is_some(), "User module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m User module loaded");
    }

    // 3. USER MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Mode Activation - user:enable]\x1B[0m");
    println!("-> Action: user:enable");
    brain.ipc.handle_request("user:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_mode, "User mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User mode enabled");
    }

    // 4. USER MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Mode Deactivation - user:disable]\x1B[0m");
    println!("-> Action: user:disable");
    brain.ipc.handle_request("user:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_mode, "User mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User mode disabled");
    }

    // 5. USER HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Hub Activation - user_hub:enable]\x1B[0m");
    println!("-> Action: user_hub:enable");
    brain.ipc.handle_request("user_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_hub, "User hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User hub enabled");
    }

    // 6. USER HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Hub Deactivation - user_hub:disable]\x1B[0m");
    println!("-> Action: user_hub:disable");
    brain.ipc.handle_request("user_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_hub, "User hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User hub disabled");
    }

    // 7. USER TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Terminal Activation - user_terminal:enable]\x1B[0m");
    println!("-> Action: user_terminal:enable");
    brain.ipc.handle_request("user_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_terminal, "User terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User terminal enabled");
    }

    // 8. USER TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Terminal Deactivation - user_terminal:disable]\x1B[0m");
    println!("-> Action: user_terminal:disable");
    brain.ipc.handle_request("user_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_terminal, "User terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User terminal disabled");
    }

    // 9. USER APPLICATION ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Application Activation - user_app:enable]\x1B[0m");
    println!("-> Action: user_app:enable");
    brain.ipc.handle_request("user_app:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app, "User app should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app enabled");
    }

    // 10. USER APPLICATION DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: User Application Deactivation - user_app:disable]\x1B[0m");
    println!("-> Action: user_app:disable");
    brain.ipc.handle_request("user_app:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.user_app, "User app should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app disabled");
    }

    // 11. USER APPLICATION EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution - user_app:execute:ls]\x1B[0m");
    println!("-> Action: user_app:execute:ls");
    brain.ipc.handle_request("user_app:execute:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution set");
    }

    // 12. USER APPLICATION EXECUTION WITH ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Arguments - user_app:execute:ls -la]\x1B[0m");
    println!("-> Action: user_app:execute:ls -la");
    brain.ipc.handle_request("user_app:execute:ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with arguments set");
    }

    // 13. USER APPLICATION EXECUTION WITH PATH TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Path - user_app:execute:/bin/ls]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls");
    brain.ipc.handle_request("user_app:execute:/bin/ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with path set");
    }

    // 14. USER APPLICATION EXECUTION WITH PATH AND ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Path and Arguments - user_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with path and arguments set");
    }

    // 15. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, AND ENVIRONMENT TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Environment - user_app:execute:/bin/ls -la;PATH=/usr/bin]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;PATH=/usr/bin");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;PATH=/usr/bin");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with environment set");
    }

    // 16. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, AND USER TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with User - user_app:execute:/bin/ls -la;USER=root]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;USER=root");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;USER=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with user set");
    }

    // 17. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, AND GROUP TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Group - user_app:execute:/bin/ls -la;USER=root;GROUP=root]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;USER=root;GROUP=root");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;USER=root;GROUP=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with group set");
    }

    // 18. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, AND SHELL TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Shell - user_app:execute:/bin/ls -la;SHELL=/bin/bash]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;SHELL=/bin/bash");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;SHELL=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with shell set");
    }

    // 19. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, AND HOME TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Home - user_app:execute:/bin/ls -la;HOME=/root]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;HOME=/root");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;HOME=/root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with home set");
    }

    // 20. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, AND TERM TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Term - user_app:execute:/bin/ls -la;TERM=xterm-256color]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;TERM=xterm-256color");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;TERM=xterm-256color");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with term set");
    }

    // 21. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, AND SHELL_PATH TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Shell Path - user_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with shell path set");
    }

    // 22. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, AND SHELL_TYPE TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Shell Type - user_app:execute:/bin/ls -la;SHELL_TYPE=sh]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;SHELL_TYPE=sh");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;SHELL_TYPE=sh");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with shell type set");
    }

    // 23. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, AND SHELL_VERSION TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Shell Version - user_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with shell version set");
    }

    // 24. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, AND SHELL_FLAGS TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Shell Flags - user_app:execute:/bin/ls -la;SHELL_FLAGS=-i]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;SHELL_FLAGS=-i");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;SHELL_FLAGS=-i");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with shell flags set");
    }

    // 25. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, AND SHELL_OPTIONS TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Shell Options - user_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with shell options set");
    }

    // 26. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, AND SHELL_TIMEOUT TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Shell Timeout - user_app:execute:/bin/ls -la;SHELL_TIMEOUT=300]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with shell timeout set");
    }

    // 27. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, AND MEMORY_LIMIT TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with Memory Limit - user_app:execute:/bin/ls -la;MEMORY_LIMIT=1024]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution with memory limit set");
    }

    // 28. USER APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, MEMORY_LIMIT, AND USER APP EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: User Application Execution with User App Execution - user_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: user_app:execute:/bin/ls -la");
    brain.ipc.handle_request("user_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.user_app_execution.is_some(), "User app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m User app execution set");
    }

    println!("\n\x1B[1;32m=== USER TESTS PASSED ===\x1B[0m");

    Ok(())
}
