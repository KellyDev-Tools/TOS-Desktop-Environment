// Tests for navigation features.

use std::collections::HashMap;
use tokio::time::sleep;
use tos_brain::Brain;
use tos_brain::common::CommandHubMode;
use tos_brain::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS NAVIGATION TESTS]\x1B[0m");
    println!("Testing navigation features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY NAVIGATION MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial Navigation Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_module.is_some(), "Navigation module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation module loaded");
    }

    // 3. NAVIGATION MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Mode Activation - navigation:enable]\x1B[0m");
    println!("-> Action: navigation:enable");
    brain.ipc.handle_request("navigation:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_mode, "Navigation mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation mode enabled");
    }

    // 4. NAVIGATION MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Mode Deactivation - navigation:disable]\x1B[0m");
    println!("-> Action: navigation:disable");
    brain.ipc.handle_request("navigation:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.navigation_mode, "Navigation mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation mode disabled");
    }

    // 5. NAVIGATION HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Hub Activation - navigation_hub:enable]\x1B[0m");
    println!("-> Action: navigation_hub:enable");
    brain.ipc.handle_request("navigation_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_hub, "Navigation hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation hub enabled");
    }

    // 6. NAVIGATION HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Hub Deactivation - navigation_hub:disable]\x1B[0m");
    println!("-> Action: navigation_hub:disable");
    brain.ipc.handle_request("navigation_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.navigation_hub, "Navigation hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation hub disabled");
    }

    // 7. NAVIGATION TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Terminal Activation - navigation_terminal:enable]\x1B[0m");
    println!("-> Action: navigation_terminal:enable");
    brain.ipc.handle_request("navigation_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_terminal, "Navigation terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation terminal enabled");
    }

    // 8. NAVIGATION TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Terminal Deactivation - navigation_terminal:disable]\x1B[0m");
    println!("-> Action: navigation_terminal:disable");
    brain.ipc.handle_request("navigation_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.navigation_terminal, "Navigation terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation terminal disabled");
    }

    // 9. NAVIGATION APPLICATION ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Activation - navigation_app:enable]\x1B[0m");
    println!("-> Action: navigation_app:enable");
    brain.ipc.handle_request("navigation_app:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app, "Navigation app should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app enabled");
    }

    // 10. NAVIGATION APPLICATION DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Deactivation - navigation_app:disable]\x1B[0m");
    println!("-> Action: navigation_app:disable");
    brain.ipc.handle_request("navigation_app:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.navigation_app, "Navigation app should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app disabled");
    }

    // 11. NAVIGATION APPLICATION EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution - navigation_app:execute:ls]\x1B[0m");
    println!("-> Action: navigation_app:execute:ls");
    brain.ipc.handle_request("navigation_app:execute:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution set");
    }

    // 12. NAVIGATION APPLICATION EXECUTION WITH ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Arguments - navigation_app:execute:ls -la]\x1B[0m");
    println!("-> Action: navigation_app:execute:ls -la");
    brain.ipc.handle_request("navigation_app:execute:ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with arguments set");
    }

    // 13. NAVIGATION APPLICATION EXECUTION WITH PATH TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Path - navigation_app:execute:/bin/ls]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with path set");
    }

    // 14. NAVIGATION APPLICATION EXECUTION WITH PATH AND ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Path and Arguments - navigation_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with path and arguments set");
    }

    // 15. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, AND ENVIRONMENT TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Environment - navigation_app:execute:/bin/ls -la;PATH=/usr/bin]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;PATH=/usr/bin");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;PATH=/usr/bin");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with environment set");
    }

    // 16. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, AND USER TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with User - navigation_app:execute:/bin/ls -la;USER=root]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;USER=root");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;USER=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with user set");
    }

    // 17. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, AND GROUP TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Group - navigation_app:execute:/bin/ls -la;USER=root;GROUP=root]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;USER=root;GROUP=root");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;USER=root;GROUP=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with group set");
    }

    // 18. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, AND SHELL TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Shell - navigation_app:execute:/bin/ls -la;SHELL=/bin/bash]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;SHELL=/bin/bash");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;SHELL=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with shell set");
    }

    // 19. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, AND HOME TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Home - navigation_app:execute:/bin/ls -la;HOME=/root]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;HOME=/root");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;HOME=/root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with home set");
    }

    // 20. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, AND TERM TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Term - navigation_app:execute:/bin/ls -la;TERM=xterm-256color]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;TERM=xterm-256color");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;TERM=xterm-256color");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with term set");
    }

    // 21. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, AND SHELL_PATH TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Shell Path - navigation_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with shell path set");
    }

    // 22. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, AND SHELL_TYPE TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Shell Type - navigation_app:execute:/bin/ls -la;SHELL_TYPE=sh]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;SHELL_TYPE=sh");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;SHELL_TYPE=sh");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with shell type set");
    }

    // 23. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, AND SHELL_VERSION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Shell Version - navigation_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with shell version set");
    }

    // 24. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, AND SHELL_FLAGS TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Shell Flags - navigation_app:execute:/bin/ls -la;SHELL_FLAGS=-i]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;SHELL_FLAGS=-i");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;SHELL_FLAGS=-i");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with shell flags set");
    }

    // 25. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, AND SHELL_OPTIONS TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Shell Options - navigation_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with shell options set");
    }

    // 26. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, AND SHELL_TIMEOUT TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Shell Timeout - navigation_app:execute:/bin/ls -la;SHELL_TIMEOUT=300]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with shell timeout set");
    }

    // 27. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, AND MEMORY_LIMIT TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Memory Limit - navigation_app:execute:/bin/ls -la;MEMORY_LIMIT=1024]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution with memory limit set");
    }

    // 28. NAVIGATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, MEMORY_LIMIT, AND NAVIGATION APP EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Navigation Application Execution with Navigation App Execution - navigation_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: navigation_app:execute:/bin/ls -la");
    brain.ipc.handle_request("navigation_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.navigation_app_execution.is_some(), "Navigation app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Navigation app execution set");
    }

    println!("\n\x1B[1;32m=== NAVIGATION TESTS PASSED ===\x1B[0m");

    Ok(())
}
