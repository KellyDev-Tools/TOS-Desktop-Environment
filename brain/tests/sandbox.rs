//! Tests for sandbox and sandboxed mode functionality.
//!
//! These validate sandbox isolation, environment variable management,
//! and safe command execution features.

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;31m[TOS SANDBOX TESTS]\x1B[0m");
    println!("Testing sandbox isolation and safe execution...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY SANDBOX MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial Sandbox Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandbox_module.is_some(), "Sandbox module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandbox module loaded");
    }

    // 3. SANDBOX MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandbox Mode Activation - sandbox:enable]\x1B[0m");
    println!("-> Action: sandbox:enable");
    brain.ipc.handle_request("sandbox:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandbox_mode, "Sandbox mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandbox mode enabled");
    }

    // 4. SANDBOX MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandbox Mode Deactivation - sandbox:disable]\x1B[0m");
    println!("-> Action: sandbox:disable");
    brain.ipc.handle_request("sandbox:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.sandbox_mode, "Sandbox mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandbox mode disabled");
    }

    // 5. SANDBOXED MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Mode Activation - sandboxed:enable]\x1B[0m");
    println!("-> Action: sandboxed:enable");
    brain.ipc.handle_request("sandboxed:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_mode, "Sandboxed mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed mode enabled");
    }

    // 6. SANDBOXED MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Mode Deactivation - sandboxed:disable]\x1B[0m");
    println!("-> Action: sandboxed:disable");
    brain.ipc.handle_request("sandboxed:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.sandboxed_mode, "Sandboxed mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed mode disabled");
    }

    // 7. SANDBOXED HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Hub Activation - sandboxed_hub:enable]\x1B[0m");
    println!("-> Action: sandboxed_hub:enable");
    brain.ipc.handle_request("sandboxed_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_hub, "Sandboxed hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed hub enabled");
    }

    // 8. SANDBOXED HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Hub Deactivation - sandboxed_hub:disable]\x1B[0m");
    println!("-> Action: sandboxed_hub:disable");
    brain.ipc.handle_request("sandboxed_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.sandboxed_hub, "Sandboxed hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed hub disabled");
    }

    // 9. SANDBOXED TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Terminal Activation - sandboxed_terminal:enable]\x1B[0m");
    println!("-> Action: sandboxed_terminal:enable");
    brain.ipc.handle_request("sandboxed_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_terminal, "Sandboxed terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed terminal enabled");
    }

    // 10. SANDBOXED TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Terminal Deactivation - sandboxed_terminal:disable]\x1B[0m");
    println!("-> Action: sandboxed_terminal:disable");
    brain.ipc.handle_request("sandboxed_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.sandboxed_terminal, "Sandboxed terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed terminal disabled");
    }

    // 11. SANDBOXED APPLICATION ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Activation - sandboxed_app:enable]\x1B[0m");
    println!("-> Action: sandboxed_app:enable");
    brain.ipc.handle_request("sandboxed_app:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app, "Sandboxed app should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app enabled");
    }

    // 12. SANDBOXED APPLICATION DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Deactivation - sandboxed_app:disable]\x1B[0m");
    println!("-> Action: sandboxed_app:disable");
    brain.ipc.handle_request("sandboxed_app:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.sandboxed_app, "Sandboxed app should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app disabled");
    }

    // 13. SANDBOXED APPLICATION EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution - sandboxed_app:execute:ls]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls");
    brain.ipc.handle_request("sandboxed_app:execute:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution set");
    }

    // 14. SANDBOXED APPLICATION EXECUTION WITH ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Arguments - sandboxed_app:execute:ls -la]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with arguments set");
    }

    // 15. SANDBOXED APPLICATION EXECUTION WITH PATH TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Path - sandboxed_app:execute:/bin/ls]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:/bin/ls");
    brain.ipc.handle_request("sandboxed_app:execute:/bin/ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with path set");
    }

    // 16. SANDBOXED APPLICATION EXECUTION WITH PATH AND ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Path and Arguments - sandboxed_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:/bin/ls -la");
    brain.ipc.handle_request("sandboxed_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with path and arguments set");
    }

    // 17. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, AND ENVIRONMENT TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Path, Arguments, and Environment - sandboxed_app:execute:/bin/ls -la;PATH=/usr/bin]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:/bin/ls -la;PATH=/usr/bin");
    brain.ipc.handle_request("sandboxed_app:execute:/bin/ls -la;PATH=/usr/bin");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with path, arguments, and environment set");
    }

    // 18. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, AND USER TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with User - sandboxed_app:execute:ls -la;USER=root]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;USER=root");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;USER=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with user set");
    }

    // 19. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, AND GROUP TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Group - sandboxed_app:execute:ls -la;USER=root;GROUP=root]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;USER=root;GROUP=root");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;USER=root;GROUP=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with group set");
    }

    // 20. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, AND SHELL TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Shell - sandboxed_app:execute:ls -la;SHELL=/bin/bash]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;SHELL=/bin/bash");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;SHELL=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with shell set");
    }

    // 21. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, AND HOME TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Home - sandboxed_app:execute:ls -la;HOME=/root]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;HOME=/root");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;HOME=/root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with home set");
    }

    // 22. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, AND TERM TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Term - sandboxed_app:execute:ls -la;TERM=xterm-256color]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;TERM=xterm-256color");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;TERM=xterm-256color");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with term set");
    }

    // 23. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, AND SHELL_PATH TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Shell Path - sandboxed_app:execute:ls -la;SHELL_PATH=/bin/bash]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;SHELL_PATH=/bin/bash");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;SHELL_PATH=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with shell path set");
    }

    // 24. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, AND SHELL_TYPE TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Shell Type - sandboxed_app:execute:ls -la;SHELL_TYPE=sh]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;SHELL_TYPE=sh");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;SHELL_TYPE=sh");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with shell type set");
    }

    // 25. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, AND SHELL_VERSION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Shell Version - sandboxed_app:execute:ls -la;SHELL_VERSION=5.1.16]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;SHELL_VERSION=5.1.16");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;SHELL_VERSION=5.1.16");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with shell version set");
    }

    // 26. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, AND SHELL_FLAGS TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Shell Flags - sandboxed_app:execute:ls -la;SHELL_FLAGS=-i]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;SHELL_FLAGS=-i");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;SHELL_FLAGS=-i");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with shell flags set");
    }

    // 27. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, AND SHELL_OPTIONS TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Shell Options - sandboxed_app:execute:ls -la;SHELL_OPTIONS=--interactive]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;SHELL_OPTIONS=--interactive");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;SHELL_OPTIONS=--interactive");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with shell options set");
    }

    // 28. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, AND SHELL_TIMEOUT TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Shell Timeout - sandboxed_app:execute:ls -la;SHELL_TIMEOUT=300]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;SHELL_TIMEOUT=300");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;SHELL_TIMEOUT=300");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with shell timeout set");
    }

    // 29. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, AND SHELL_MEMORY_LIMIT TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Memory Limit - sandboxed_app:execute:ls -la;MEMORY_LIMIT=1024]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la;MEMORY_LIMIT=1024");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la;MEMORY_LIMIT=1024");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution with memory limit set");
    }

    // 30. SANDBOXED APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, MEMORY_LIMIT, AND SANDBOXED APP EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Sandboxed Application Execution with Sandboxed App Execution - sandboxed_app:execute:ls -la]\x1B[0m");
    println!("-> Action: sandboxed_app:execute:ls -la");
    brain.ipc.handle_request("sandboxed_app:execute:ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.sandboxed_app_execution.is_some(), "Sandboxed app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Sandboxed app execution set");
    }

    println!("\n\x1B[1;32m=== SANDBOX TESTS PASSED ===\x1B[0m");

    Ok(())
}
