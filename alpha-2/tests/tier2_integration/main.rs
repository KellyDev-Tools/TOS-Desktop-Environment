use tos_alpha2::brain::shell::ShellApi;
use tos_alpha2::common::TosState;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_shell_output_and_osc_priority() {
    let _ = tracing_subscriber::fmt::try_init();
    let state = Arc::new(Mutex::new(TosState::default()));
    let mut shell = ShellApi::new(state.clone()).expect("Failed to spawn shell");

    // Send a command that emits the OSC 9012 sequence for priority 2 (Warning)
    // printf is more portable than echo -e
    let cmd = "printf '\\033]9012;2\\007SENTINEL_OUTPUT\\n'\n";
    shell.write(cmd).expect("Failed to write to shell");

    // Wait for the shell to process and the reader thread to update state
    let mut found = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(100)).await;
        let state_lock = state.lock().unwrap();
        let hub = &state_lock.sectors[0].hubs[0];
        
        // Match only the output line, avoiding the echoed command line
        if let Some(_line) = hub.terminal_output.iter().find(|l| l.text.contains("SENTINEL_OUTPUT") && l.priority == 2) {
            found = true;
            break;
        }
    }

    assert!(found, "Did not receive priority-tagged output from shell");
}

#[tokio::test]
async fn test_terminal_buffer_fifo() {
    let state = Arc::new(Mutex::new(TosState::default()));
    
    // Set a very small buffer limit for the test
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.sectors[0].hubs[0].buffer_limit = 3;
    }
    
    let mut shell = ShellApi::new(state.clone()).expect("Failed to spawn shell");

    // Send 5 lines
    for i in 1..=5 {
        shell.write(&format!("echo line{}\n", i)).expect("Write failed");
        sleep(Duration::from_millis(50)).await;
    }

    sleep(Duration::from_millis(500)).await;

    let state_lock = state.lock().unwrap();
    let hub = &state_lock.sectors[0].hubs[0];
    
    // Buffer should only contain the last 3 lines
    assert!(hub.terminal_output.len() <= 3);
    if !hub.terminal_output.is_empty() {
        // The last line should be line5
        assert!(hub.terminal_output.last().unwrap().text.contains("line5"));
    }
}
