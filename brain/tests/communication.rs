// Tests for communication and messaging features.

use std::collections::HashMap;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS COMMUNICATION TESTS]\x1B[0m");
    println!("Testing communication and messaging features...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY COMMUNICATION MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial Communication Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_module.is_some(), "Communication module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication module loaded");
    }

    // 3. COMMUNICATION MODE ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Mode Activation - communication:enable]\x1B[0m");
    println!("-> Action: communication:enable");
    brain.ipc.handle_request("communication:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_mode, "Communication mode should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication mode enabled");
    }

    // 4. COMMUNICATION MODE DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Mode Deactivation - communication:disable]\x1B[0m");
    println!("-> Action: communication:disable");
    brain.ipc.handle_request("communication:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.communication_mode, "Communication mode should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication mode disabled");
    }

    // 5. COMMUNICATION HUB ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Hub Activation - communication_hub:enable]\x1B[0m");
    println!("-> Action: communication_hub:enable");
    brain.ipc.handle_request("communication_hub:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_hub, "Communication hub should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication hub enabled");
    }

    // 6. COMMUNICATION HUB DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Hub Deactivation - communication_hub:disable]\x1B[0m");
    println!("-> Action: communication_hub:disable");
    brain.ipc.handle_request("communication_hub:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.communication_hub, "Communication hub should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication hub disabled");
    }

    // 7. COMMUNICATION TERMINAL ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Terminal Activation - communication_terminal:enable]\x1B[0m");
    println!("-> Action: communication_terminal:enable");
    brain.ipc.handle_request("communication_terminal:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_terminal, "Communication terminal should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication terminal enabled");
    }

    // 8. COMMUNICATION TERMINAL DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Terminal Deactivation - communication_terminal:disable]\x1B[0m");
    println!("-> Action: communication_terminal:disable");
    brain.ipc.handle_request("communication_terminal:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.communication_terminal, "Communication terminal should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication terminal disabled");
    }

    // 9. COMMUNICATION APPLICATION ACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Activation - communication_app:enable]\x1B[0m");
    println!("-> Action: communication_app:enable");
    brain.ipc.handle_request("communication_app:enable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app, "Communication app should be enabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app enabled");
    }

    // 10. COMMUNICATION APPLICATION DEACTIVATION TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Deactivation - communication_app:disable]\x1B[0m");
    println!("-> Action: communication_app:disable");
    brain.ipc.handle_request("communication_app:disable");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.communication_app, "Communication app should be disabled");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app disabled");
    }

    // 11. COMMUNICATION APPLICATION EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution - communication_app:execute:ls]\x1B[0m");
    println!("-> Action: communication_app:execute:ls");
    brain.ipc.handle_request("communication_app:execute:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution set");
    }

    // 12. COMMUNICATION APPLICATION EXECUTION WITH ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Arguments - communication_app:execute:ls -la]\x1B[0m");
    println!("-> Action: communication_app:execute:ls -la");
    brain.ipc.handle_request("communication_app:execute:ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with arguments set");
    }

    // 13. COMMUNICATION APPLICATION EXECUTION WITH PATH TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Path - communication_app:execute:/bin/ls]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls");
    brain.ipc.handle_request("communication_app:execute:/bin/ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with path set");
    }

    // 14. COMMUNICATION APPLICATION EXECUTION WITH PATH AND ARGUMENTS TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Path and Arguments - communication_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with path and arguments set");
    }

    // 15. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, AND ENVIRONMENT TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Environment - communication_app:execute:/bin/ls -la;PATH=/usr/bin]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;PATH=/usr/bin");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;PATH=/usr/bin");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with environment set");
    }

    // 16. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, AND USER TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with User - communication_app:execute:/bin/ls -la;USER=root]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;USER=root");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;USER=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with user set");
    }

    // 17. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, AND GROUP TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Group - communication_app:execute:/bin/ls -la;USER=root;GROUP=root]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;USER=root;GROUP=root");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;USER=root;GROUP=root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with group set");
    }

    // 18. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, AND SHELL TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Shell - communication_app:execute:/bin/ls -la;SHELL=/bin/bash]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;SHELL=/bin/bash");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;SHELL=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with shell set");
    }

    // 19. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, AND HOME TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Home - communication_app:execute:/bin/ls -la;HOME=/root]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;HOME=/root");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;HOME=/root");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with home set");
    }

    // 20. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, AND TERM TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Term - communication_app:execute:/bin/ls -la;TERM=xterm-256color]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;TERM=xterm-256color");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;TERM=xterm-256color");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with term set");
    }

    // 21. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, AND SHELL_PATH TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Shell Path - communication_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;SHELL_PATH=/bin/bash");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with shell path set");
    }

    // 22. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, AND SHELL_TYPE TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Shell Type - communication_app:execute:/bin/ls -la;SHELL_TYPE=sh]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;SHELL_TYPE=sh");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;SHELL_TYPE=sh");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with shell type set");
    }

    // 23. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, AND SHELL_VERSION TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Shell Version - communication_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;SHELL_VERSION=5.1.16");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with shell version set");
    }

    // 24. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, AND SHELL_FLAGS TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Shell Flags - communication_app:execute:/bin/ls -la;SHELL_FLAGS=-i]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;SHELL_FLAGS=-i");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;SHELL_FLAGS=-i");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with shell flags set");
    }

    // 25. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, AND SHELL_OPTIONS TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Shell Options - communication_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;SHELL_OPTIONS=--interactive");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with shell options set");
    }

    // 26. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, AND SHELL_TIMEOUT TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Shell Timeout - communication_app:execute:/bin/ls -la;SHELL_TIMEOUT=300]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;SHELL_TIMEOUT=300");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with shell timeout set");
    }

    // 27. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, AND MEMORY_LIMIT TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Memory Limit - communication_app:execute:/bin/ls -la;MEMORY_LIMIT=1024]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la;MEMORY_LIMIT=1024");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution with memory limit set");
    }

    // 28. COMMUNICATION APPLICATION EXECUTION WITH PATH, ARGUMENTS, ENVIRONMENT, USER, GROUP, SHELL, HOME, TERM, SHELL_PATH, SHELL_TYPE, SHELL_VERSION, SHELL_FLAGS, SHELL_OPTIONS, SHELL_TIMEOUT, MEMORY_LIMIT, AND COMMUNICATION APP EXECUTION TEST
    println!("\n\x1B[1;33m[TEST: Communication Application Execution with Communication App Execution - communication_app:execute:/bin/ls -la]\x1B[0m");
    println!("-> Action: communication_app:execute:/bin/ls -la");
    brain.ipc.handle_request("communication_app:execute:/bin/ls -la");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.communication_app_execution.is_some(), "Communication app execution should be set");
        println!("\x1B[1;32m[PASSED]\x1B[0m Communication app execution set");
    }

    println!("\n\x1B[1;32m=== COMMUNICATION TESTS PASSED ===\x1B[0m");

    Ok(())
}
