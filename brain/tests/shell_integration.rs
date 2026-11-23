use tos_common::brain::Brain;
use std::time::Duration;
use tokio::time::sleep;

/// Wait until the shell is actually ready to run a command. The
/// readiness signal is the session's grid: the shell's first prompt
/// is parsed into it as display state (terminal_output only fills
/// on complete lines, and a prompt has no newline yet). A fixed
/// 500ms sleep raced shell startup on loaded CI runners — the
/// printf went in before the shell was reading and the old 2s poll
/// expired before the OSC sequence came back (both tests failed
/// together in one CI re-run with identical code green in the
/// previous run).
async fn wait_for_shell_ready(brain: &Brain) {
    for _ in 0..100 {
        {
            let shell = brain.shell.lock().unwrap();
            if shell.display_seq() > 0 {
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

/// Type a command into the shell exactly as a face does: the
/// text as a text event, the newline as an Enter key event —
/// the Brain encodes both (Architecture §26.3).
fn type_command(brain: &Brain, command: &str) {
    let text = command.strip_suffix('\n').unwrap_or(command);
    if !text.is_empty() {
        let payload = serde_json::json!({ "text": text }).to_string();
        brain.ipc.handle_request(&format!("terminal_text:{payload}"));
    }
    if command.ends_with('\n') {
        brain
            .ipc
            .handle_request("terminal_key:{\"key\":\"Enter\",\"code\":\"Enter\"}");
    }
}

#[tokio::test]
async fn test_command_completion_syncs_cwd_and_listing() {
    // Phase 1.6: with a stock shell (no OSC 7, no TOS integration),
    // a plain `cd` must still reach the Brain's model: the PTY
    // completion watcher syncs the hub cwd from the shell process
    // (/proc/<pid>/cwd) and re-issues the listing as a new
    // generation. A content-free command (`pwd`) re-issues too —
    // every completed command is a new chip request.
    let brain = Brain::new().expect("Failed to initialize Brain");
    wait_for_shell_ready(&brain).await;

    let (start_dir, start_gen) = {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        (
            hub.current_directory.clone(),
            hub.shell_listing.as_ref().map_or(0, |l| l.generation),
        )
    };
    assert!(start_gen >= 1, "startup listing should carry a generation");
    assert_ne!(start_dir, std::path::PathBuf::from("/tmp"));

    type_command(&brain, "cd /tmp\n");

    let mut synced = false;
    for _ in 0..100 {
        sleep(Duration::from_millis(100)).await;
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        if hub.current_directory == std::path::PathBuf::from("/tmp")
            && hub.shell_listing.as_ref().map(|l| l.path.as_str()) == Some("/tmp")
            && hub.shell_listing.as_ref().map_or(0, |l| l.generation) > start_gen
        {
            synced = true;
            break;
        }
    }
    assert!(synced, "command completion did not sync cwd/listing from the shell");

    // pwd changes nothing — and still produces a new generation.
    let gen_after_cd = {
        let state = brain.state.lock().unwrap();
        state.sectors[0].hubs[0].shell_listing.as_ref().map_or(0, |l| l.generation)
    };
    type_command(&brain, "pwd\n");
    let mut reissued = false;
    for _ in 0..100 {
        sleep(Duration::from_millis(100)).await;
        let state = brain.state.lock().unwrap();
        if state.sectors[0].hubs[0].shell_listing.as_ref().map_or(0, |l| l.generation) > gen_after_cd {
            reissued = true;
            break;
        }
    }
    assert!(reissued, "pwd did not produce a new listing generation");
}
