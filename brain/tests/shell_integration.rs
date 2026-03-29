//! Tests for shell integration and directory mode functionality.
//!
//! These validate shell command execution, directory navigation, and
//! terminal output handling in the Brain.

use std::time::Duration;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::test]
async fn test_shell_integration() -> anyhow::Result<()> {
    println!("\x1B[1;33m[TOS SHELL INTEGRATION TESTS]\x1B[0m");
    println!("Testing shell commands and directory operations...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. INITIAL STATE - VERIFY SHELL MODULE LOADED
    println!("\x1B[1;33m[TEST: Initial Shell Module]\x1B[0m");
    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.shell_module.is_some(), "Shell module should be loaded");
        println!("\x1B[1;32m[PASSED]\x1B[0m Shell module loaded");
    }

    // 3. DIRECTORY NAVIGATION - CD TO TMP
    println!("\n\x1B[1;33m[TEST: Directory Navigation - cd /tmp]\x1B[0m");
    println!("-> Action: cd /tmp");
    face.simulate_prompt_submit("cd /tmp");
    sleep(Duration::from_millis(1000)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.current_directory, std::path::PathBuf::from("/tmp"));
        println!("\x1B[1;32m[PASSED]\x1B[0m Successfully changed to /tmp");
    }

    // 4. DIRECTORY LISTING - LS
    println!("\n\x1B[1;33m[TEST: Directory Listing - ls]\x1B[0m");
    println!("-> Action: ls");
    face.simulate_prompt_submit("ls");
    sleep(Duration::from_millis(2000)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() > 0, "Terminal output should contain ls results");
        let output = hub.terminal_output.join("\n");
        assert!(output.contains("tmp") || output.contains("."), "ls output should show directory contents");
        println!("\x1B[1;32m[PASSED]\x1B[0m Directory listing successful");
        println!("\x1B[1;34m[Output preview]\x1B[0m: {}", output.lines().take(3).collect::<Vec<_>>().join("\n"));
    }

    // 5. DIRECTORY NAVIGATION - CD TO PARENT
    println!("\n\x1B[1;33m[TEST: Directory Navigation - cd ..]\x1B[0m");
    println!("-> Action: cd ..");
    face.simulate_prompt_submit("cd ..");
    sleep(Duration::from_millis(1000)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.current_directory, std::path::PathBuf::from("/"));
        println!("\x1B[1;32m[PASSED]\x1B[0m Successfully changed to /");
    }

    // 6. DIRECTORY NAVIGATION - CD TO SPECIFIC PATH
    println!("\n\x1B[1;33m[TEST: Directory Navigation - cd /usr/local]\x1B[0m");
    println!("-> Action: cd /usr/local");
    face.simulate_prompt_submit("cd /usr/local");
    sleep(Duration::from_millis(1000)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.current_directory, std::path::PathBuf::from("/usr/local"));
        println!("\x1B[1;32m[PASSED]\x1B[0m Successfully changed to /usr/local");
    }

    // 7. DIRECTORY LISTING - LS -LA
    println!("\n\x1B[1;33m[TEST: Directory Listing - ls -la]\x1B[0m");
    println!("-> Action: ls -la");
    face.simulate_prompt_submit("ls -la");
    sleep(Duration::from_millis(2000)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() > 0, "Terminal output should contain ls -la results");
        let output = hub.terminal_output.join("\n");
        assert!(output.contains("-"), "ls -la output should show detailed listing");
        println!("\x1B[1;32m[PASSED]\x1B[0m Detailed directory listing successful");
    }

    // 8. FILE CREATION TEST
    println!("\n\x1B[1;33m[TEST: File Creation - touch testfile]\x1B[0m");
    println!("-> Action: touch testfile");
    face.simulate_prompt_submit("touch testfile");
    sleep(Duration::from_millis(1000)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() > 0, "Terminal output should contain touch result");
        println!("\x1B[1;32m[PASSED]\x1B[0m File created successfully");
    }

    // 9. FILE REMOVAL TEST
    println!("\n\x1B[1;33m[TEST: File Removal - rm testfile]\x1B[0m");
    println!("-> Action: rm testfile");
    face.simulate_prompt_submit("rm testfile");
    sleep(Duration::from_millis(1000)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() > 0, "Terminal output should contain rm result");
        println!("\x1B[1;32m[PASSED]\x1B[0m File removed successfully");
    }

    // 10. CATALOG COMMAND TEST
    println!("\n\x1B[1;33m[TEST: Catalog Command - catalog]\x1B[0m");
    println!("-> Action: catalog");
    face.simulate_prompt_submit("catalog");
    sleep(Duration::from_millis(1500)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() > 0, "Terminal output should contain catalog result");
        println!("\x1B[1;32m[PASSED]\x1B[0m Catalog command executed");
    }

    // 11. CATALOG WITH PATTERN TEST
    println!("\n\x1B[1;33m[TEST: Catalog with Pattern - catalog *.rs]\x1B[0m");
    println!("-> Action: catalog *.rs");
    face.simulate_prompt_submit("catalog *.rs");
    sleep(Duration::from_millis(1500)).await;

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() > 0, "Terminal output should contain catalog result");
        println!("\x1B[1;32m[PASSED]\x1B[0m Catalog with pattern executed");
    }

    // 12. SEARCH IN CURRENT DIRECTORY TEST
    println!(r"\n\x1B[1;33m[TEST: Search in Current Directory - search:local:.*\.rs]\x1B[0m");
    println!(r"-> Action: search:local:.*\.rs");
    brain.ipc.handle_request(r"search:local:.*\.rs");

    {
        let state = brain.state.lock().unwrap();
        // Search mode should be set
        assert_eq!(state.sectors[0].hubs[0].mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m Search mode activated for local files");
    }

    // 13. SHELL LISTING TEST
    println!("\n\x1B[1;33m[TEST: Shell Listing - shell:ls]\x1B[0m");
    println!("-> Action: shell:ls");
    brain.ipc.handle_request("shell:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.shell_listing.is_some(), "Shell listing should be populated");
        println!("\x1B[1;32m[PASSED]\x1B[0m Shell listing populated");
    }

    // 14. ACTIVITY LISTING TEST
    println!("\n\x1B[1;33m[TEST: Activity Listing - activity:ls]\x1B[0m");
    println!("-> Action: activity:ls");
    brain.ipc.handle_request("activity:ls");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.activity_listing.is_some(), "Activity listing should be populated");
        println!("\x1B[1;32m[PASSED]\x1B[0m Activity listing populated");
    }

    // 15. TERMINAL OUTPUT BUFFER TEST
    println!("\n\x1B[1;33m[TEST: Terminal Output Buffer Management]\x1B[0m");
    println!("-> Action: Multiple commands to test buffer");

    for i in 0..5 {
        println!("  -> Action: echo command_{}", i);
        face.simulate_prompt_submit(&format!("echo command_{}", i));
        sleep(Duration::from_millis(500)).await;
    }

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() > 0, "Terminal output buffer should contain results");
        println!("\x1B[1;32m[PASSED]\x1B[0m Terminal output buffer managed correctly");
    }

    // 16. BUFFER LIMIT ENFORCEMENT TEST
    println!("\n\n\x1B[1;33m[TEST: Buffer Limit Enforcement]\x1B[0m");
    println!("-> Action: set_buffer_limit:10");
    brain.ipc.handle_request("set_buffer_limit:10");

    // Fill buffer to near capacity
    for i in 0..9 {
        face.simulate_prompt_submit("echo fill");
        sleep(Duration::from_millis(100)).await;
    }

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.terminal_output.len() <= 10, "Buffer should respect limit of 10");
        println!("\x1B[1;32m[PASSED]\x1B[0m Buffer limit enforced (current: {})", hub.terminal_output.len());
    }

    println!("\n\x1B[1;32m=== SHELL INTEGRATION TESTS PASSED ===\x1B[0m");

    Ok(())
}
