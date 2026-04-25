use tos_common::brain::Brain;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_shell_cwd_osc_update() {
    let _ = tracing_subscriber::fmt().with_env_filter("debug").try_init();
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // Wait for shell to stabilize
    sleep(Duration::from_millis(500)).await;
    
    let cmd = "printf '\\033]7;file://localhost/tmp\\007\\n'\n";
    
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
    
    let cmd = "printf '\\033]50;3\\007Critical Alert\\n'\n";
    
    // Wait for shell to stabilize
    sleep(Duration::from_millis(500)).await;
    
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
