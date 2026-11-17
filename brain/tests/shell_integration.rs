use tos_common::brain::Brain;
use std::time::Duration;
use tokio::time::sleep;

/// Wait until the shell is actually ready to run a command. The
/// readiness signal is the PTY scrollback: the shell's first prompt
/// lands there as raw bytes (terminal_output only fills on complete
/// lines, and a prompt has no newline yet). A fixed 500ms sleep raced
/// shell startup on loaded CI runners — the printf went in before
/// the shell was reading and the old 2s poll expired before the OSC
/// sequence came back (both tests failed together in one CI re-run
/// with identical code green in the previous run).
async fn wait_for_shell_ready(brain: &Brain) {
    for _ in 0..100 {
        {
            let shell = brain.shell.lock().unwrap();
            if !shell.scrollback_snapshot().is_empty() {
                return;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
    panic!("Shell produced no output within 10s");
}

#[tokio::test]
async fn test_shell_cwd_osc_update() {
    let _ = tracing_subscriber::fmt().with_env_filter("debug").try_init();
    let brain = Brain::new().expect("Failed to initialize Brain");

    wait_for_shell_ready(&brain).await;

    let cmd = "printf '\\033]7;file://localhost/tmp\\007\\n'\n";

    brain.ipc.handle_request(&format!("prompt_submit:{}", cmd));

    // Give it some time to process the output
    let mut success = false;
    for _ in 0..100 {
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

    wait_for_shell_ready(&brain).await;

    brain.ipc.handle_request(&format!("prompt_submit:{}", cmd));

    let mut success = false;
    for _ in 0..100 {
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
