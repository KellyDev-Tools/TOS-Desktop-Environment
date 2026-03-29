// Tests for messaging features.

use std::collections::HashMap;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS MESSAGING TESTS]\x1B[0m");
    println!("Testing messaging features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY MESSAGING MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial Messaging Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_module.is_some(), "Messaging module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging module loaded");
    }

    // 3. MESSAGING MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Mode Activation - messaging:enable]\x1B[0m");
    println!("-> Action: messaging:enable");
    brain.ipc.handle_request("messaging:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_mode, "Messaging mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging mode enabled");
    }

    // 4. MESSAGING MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Mode Deactivation - messaging:disable]\x1B[0m");
    println!("-> Action: messaging:disable");
    brain.ipc.handle_request("messaging:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.messaging_mode, "Messaging mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging mode disabled");
    }

    // 5. MESSAGING HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Hub Activation - messaging_hub:enable]\x1B[0m");
    println!("-> Action: messaging_hub:enable");
    brain.ipc.handle_request("messaging_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_hub, "Messaging hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging hub enabled");
    }

    // 6. MESSAGING HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Hub Deactivation - messaging_hub:disable]\x1B[0m");
    println!("-> Action: messaging_hub:disable");
    brain.ipc.handle_request("messaging_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.messaging_hub, "Messaging hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging hub disabled");
    }

    // 7. MESSAGING TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Terminal Activation - messaging_terminal:enable]\x1B[0m");
    println!("-> Action: messaging_terminal:enable");
    brain.ipc.handle_request("messaging_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_terminal, "Messaging terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging terminal enabled");
    }

    // 8. MESSAGING TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Terminal Deactivation - messaging_terminal:disable]\x1B[0m");
    println!("-> Action: messaging_terminal:disable");
    brain.ipc.handle_request("messaging_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.messaging_terminal, "Messaging terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging terminal disabled");
    }

    // 9. MESSAGING APPLICATION ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Activation - messaging_app:enable]\x1B[0m");
    println!("-> Action: messaging_app:enable");
    brain.ipc.handle_request("messaging_app:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app, "Messaging app should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app enabled");
    }

    // 10. MESSAGING APPLICATION DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Deactivation - messaging_app:disable]\x1B[0m");
    println!("-> Action: messaging_app:disable");
    brain.ipc.handle_request("messaging_app:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.messaging_app, "Messaging app should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app disabled");
    }

    // 11. MESSAGING APPLICATION EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution - messaging_app:execute:ls]\x1B[0m");
    println!("-> Action: messaging_app:execute:ls");
    brain.ipc.handle_request("messaging_app:execute:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution set");
    }

    // 12. MESSAGING APPLICATION EXECUTION WITH ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Arguments - messaging_app:execute:ls -la]\x1B[0m");
    println!("-> Action: messaging_app:execute:ls -la");
    brain.ipc.handle_request("messaging_app:execute:ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with arguments set");
    }

    // 13. MESSAGING APPLICATION EXECUTION WITH PATH TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Path - messaging_app:execute:/bin/ls]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with path set");
    }

    // 14. MESSAGING APPLICATION EXECUTION WITH PATH AND ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Path and Arguments - messaging_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with path and arguments set");
    }

    // 15. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, AND ENVIRONMENT TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Environment - messaging_app:execute:/bin/ls -la;PATH=/usr/bin]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;PATH=/usr/bin");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;PATH=/usr/bin");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with environment set");
    }

    // 16. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, AND USER TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with User - messaging_app:execute:/bin/ls -la;USER=root]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;USER=root");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;USER=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with user set");
    }

    // 17. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, AND GROUP TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Group - messaging_app:execute:/bin/ls -la;USER=root;GROUP=root]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;USER=root;GROUP=root");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;USER=root;GROUP=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with group set");
    }

    // 18. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, AND SHELL TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Shell - messaging_app:execute:/bin/ls -la;SHELL=/bin/bash]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;SHELL=/bin/bash");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;SHELL=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with shell set");
    }

    // 19. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, AND HOME TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Home - messaging_app:execute:/bin/ls -la;HOME=/root]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;HOME=/root");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;HOME=/root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with home set");
    }

    // 20. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, AND TERM TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Term - messaging_app:execute:/bin/ls -la;TERM=xterm-256color]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;TERM=xterm-256color");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;TERM=xterm-256color");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with term set");
    }

    // 21. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, AND SHELL_PATH TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Shell Path - messaging_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with shell path set");
    }

    // 22. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, AND SHELL_TYPE TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Shell Type - messaging_app:execute:/bin/ls -la;SHELL_TYPE=sh]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;SHELL_TYPE=sh");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;SHELL_TYPE=sh");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with shell type set");
    }

    // 23. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, AND SHELL_VERSION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Shell Version - messaging_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with shell version set");
    }

    // 24. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, AND SHELL_FLAGS TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Shell Flags - messaging_app:execute:/bin/ls -la;SHELL_FLAGS=-i]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;SHELL_FLAGS=-i");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;SHELL_FLAGS=-i");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with shell flags set");
    }

    // 25. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, AND SHELL_OPTIONS TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Shell Options - messaging_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with shell options set");
    }

    // 26. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, AND SHELL_TIMEOUT TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Shell Timeout - messaging_app:execute:/bin/ls -la;SHELL_TIMEOUT=300]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with shell timeout set");
    }

    // 27. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, AND MEMORY_LIMIT TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Memory Limit - messaging_app:execute:/bin/ls -la;MEMORY_LIMIT=1024]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution with memory limit set");
    }

    // 28. MESSAGING APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, MEMORY_LIMIT, AND MESSAGING APP EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Messaging Application Execution with Messaging App Execution - messaging_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: messaging_app:execute:/bin/ls -la");
    brain.ipc.handle_request("messaging_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.messaging_app_execution.is_some(), "Messaging app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Messaging app execution set");
    }

    println!("\n\x1B[1;32m=== MESSAGING TESTS PASSED ===\x1B[0m");

    Ok(())
}
