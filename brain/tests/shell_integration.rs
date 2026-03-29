use tos_common::brain::Brain;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_shell_cwd_osc_update() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // Initial CWD
    let initial_cwd = {
        let state = brain.state.lock().unwrap();
        state.sectors[0].hubs[0].current_directory.clone()
    };
    
    // We send a command that should emit an OSC sequence for CWD if the shell supports it.
    // However, since we are using a real PTY, it depends on the shell (bash/fish/zsh).
    // To be shell-agnostic in the test, we can manually send the OSC sequence to the PTY's internal reader 
    // if we had access to it, but we don't easily.
    // Instead, we can try to 'echo' the OSC sequence and see if our parser picks it up from the STDOUT.
    
    let osc_cwd = format!("\x1b]50;2;file://localhost/tmp\x07");
    let cmd = format!("echo -n '{}'\n", osc_cwd);
    
    brain.ipc.handle_request(&format!("prompt_submit:{}", cmd));
    
    // Give it some time to process the output
    let mut success = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let state = brain.state.lock().unwrap();
        if state.sectors[0].hubs[0].current_directory.to_string_lossy().contains("tmp") {
            success = true;
            break;
        }
    }
    
    assert!(success, "CWD did not update via OSC sequence");
}

#[tokio::test]
async fn test_shell_priority_osc_update() {
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    let osc_priority = "\x1b]50;3\x07Critical Alert";
    let cmd = format!("echo -n '{}'\n", osc_priority);
    
    brain.ipc.handle_request(&format!("prompt_submit:{}", cmd));
    
    let mut success = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        if hub.terminal_output.iter().any(|line| line.text.contains("Critical Alert") && line.priority == 3) {
            success = true;
            break;
        }
    }
    
    assert!(success, "Priority did not update via OSC sequence");
}
