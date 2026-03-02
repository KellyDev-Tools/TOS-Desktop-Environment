use tos_alpha2::brain::shell::ShellApi;
use tos_alpha2::common::TosState;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_shell_output_and_osc_priority() {
    let _ = tracing_subscriber::fmt::try_init();
    let state_val = TosState::default();
    let sid = state_val.sectors[0].id;
    let hid = state_val.sectors[0].hubs[0].id;
    let state = Arc::new(Mutex::new(state_val));
    let mut shell = ShellApi::new(state.clone(), sid, hid).expect("Failed to spawn shell");

    // Send a command that emits the OSC 9012 sequence for priority 2 (Warning)
    let cmd = "printf '\\033]9012;2\\007SENTINEL_OUTPUT\\n'\n";
    shell.write(cmd).expect("Failed to write to shell");

    let mut found = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(100)).await;
        let state_lock = state.lock().unwrap();
        let hub = &state_lock.sectors[0].hubs[0];
        
        if let Some(_line) = hub.terminal_output.iter().find(|l| l.text.contains("SENTINEL_OUTPUT") && l.priority == 2) {
            found = true;
            break;
        }
    }

    assert!(found, "Did not receive priority-tagged output from shell");
}

#[tokio::test]
async fn test_terminal_buffer_fifo() {
    let state_val = TosState::default();
    let sid = state_val.sectors[0].id;
    let hid = state_val.sectors[0].hubs[0].id;
    let state = Arc::new(Mutex::new(state_val));
    
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.sectors[0].hubs[0].buffer_limit = 3;
    }
    
    let mut shell = ShellApi::new(state.clone(), sid, hid).expect("Failed to spawn shell");

    for i in 1..=5 {
        shell.write(&format!("echo line{}\n", i)).expect("Write failed");
        sleep(Duration::from_millis(50)).await;
    }

    sleep(Duration::from_millis(500)).await;

    let state_lock = state.lock().unwrap();
    let hub = &state_lock.sectors[0].hubs[0];
    
    assert!(hub.terminal_output.len() <= 3);
    if !hub.terminal_output.is_empty() {
        assert!(hub.terminal_output.last().unwrap().text.contains("line5"));
    }
}

#[tokio::test]
async fn test_shell_cwd_tracking() {
    let state_val = TosState::default();
    let sid = state_val.sectors[0].id;
    let hid = state_val.sectors[0].hubs[0].id;
    let state = Arc::new(Mutex::new(state_val));
    let mut shell = ShellApi::new(state.clone(), sid, hid).expect("Failed to spawn shell");

    let test_path = "/tmp/tos_test_dir";
    let cmd = format!("printf '\\033]9003;{}\\007\\n'\n", test_path);
    shell.write(&cmd).expect("Write failed");

    let mut found = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let state_lock = state.lock().unwrap();
        if state_lock.sectors[0].hubs[0].current_directory.to_string_lossy() == test_path {
            found = true;
            break;
        }
    }

    assert!(found, "CWD did not update from OSC 9003");
}

#[tokio::test]
async fn test_shell_command_result() {
    let state_val = TosState::default();
    let sid = state_val.sectors[0].id;
    let hid = state_val.sectors[0].hubs[0].id;
    let state = Arc::new(Mutex::new(state_val));
    let mut shell = ShellApi::new(state.clone(), sid, hid).expect("Failed to spawn shell");

    let cmd = "printf '\\033]9002;ls;0;dGVzdCBvdXRwdXQ=\\007\\n'\n";
    shell.write(cmd).expect("Write failed");

    let mut found = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let state_lock = state.lock().unwrap();
        if state_lock.sectors[0].hubs[0].terminal_output.len() > 0 {
            found = true;
            break;
        }
    }
    assert!(found);
}

#[tokio::test]
async fn test_shell_directory_listing() {
    let state_val = TosState::default();
    let sid = state_val.sectors[0].id;
    let hid = state_val.sectors[0].hubs[0].id;
    let state = Arc::new(Mutex::new(state_val));
    let mut shell = ShellApi::new(state.clone(), sid, hid).expect("Failed to spawn shell");

    let b64_json = "eyJwYXRoIjoiL3Rlc3QiLCJlbnRyaWVzIjpbeyJuYW1lIjoiZmlsZS50eHQiLCJpc19kaXIiOmZhbHNlLCJzaXplIjoxMDB9XX0=";
    let cmd = format!("printf '\\033]9001;/test;{}\\007\\n'\n", b64_json);
    shell.write(&cmd).expect("Write failed");

    let mut found = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let state_lock = state.lock().unwrap();
        if let Some(listing) = &state_lock.sectors[0].hubs[0].shell_listing {
            if listing.path == "/test" && listing.entries[0].name == "file.txt" {
                found = true;
                break;
            }
        }
    }

    assert!(found, "Directory listing did not update from OSC 9001");
}

#[tokio::test]
async fn test_remote_session_disconnection() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut state_val = TosState::default();
    state_val.sectors[0].is_remote = true;
    let sid = state_val.sectors[0].id;
    let hid = state_val.sectors[0].hubs[0].id;
    let state = Arc::new(Mutex::new(state_val));
    
    let mut shell = ShellApi::new(state.clone(), sid, hid).expect("Failed to spawn shell");
    
    // Simulate terminal closure (send exit)
    shell.write("exit\n").expect("Write failed");
    drop(shell);
    
    // Verify disconnected state immediately
    {
        let mut found = false;
        for _ in 0..20 {
            sleep(Duration::from_millis(100)).await;
            let state_lock = state.lock().unwrap();
            if state_lock.sectors.first().map(|s| s.disconnected).unwrap_or(false) {
                found = true;
                break;
            }
        }
        assert!(found, "Sector should be marked disconnected");
    }
    
    // Verify auto-close after 5 seconds
    sleep(Duration::from_secs(6)).await;
    {
        let state_lock = state.lock().unwrap();
        assert!(state_lock.sectors.is_empty(), "Sector should be auto-closed after 5 seconds");
    }
}

#[tokio::test]
async fn test_dangerous_command_interception() {
    use tos_alpha2::brain::ipc_handler::IpcHandler;
    use tos_alpha2::services::ServiceManager;
    let state_val = TosState::default();
    let sid = state_val.sectors[0].id;
    let hid = state_val.sectors[0].hubs[0].id;
    let state = Arc::new(Mutex::new(state_val));
    let shell = Arc::new(Mutex::new(ShellApi::new(state.clone(), sid, hid).unwrap()));
    let services = Arc::new(ServiceManager::new());
    let ipc = IpcHandler::new(state.clone(), shell.clone(), services);

    // 1. Submit dangerous command
    ipc.handle_request("prompt_submit:rm -rf /");

    // 2. Verify it's pending confirmation
    let conf_id = {
        let state_lock = state.lock().unwrap();
        let conf = state_lock.pending_confirmation.as_ref().expect("Should have pending confirmation");
        assert!(conf.message.contains("DANGEROUS"));
        conf.id
    };

    // 3. Simulate partial slider progress (should not execute)
    ipc.handle_request(&format!("update_confirmation_progress:{};0.5", conf_id));
    {
        let state_lock = state.lock().unwrap();
        assert!(state_lock.pending_confirmation.is_some());
    }

    // 4. Simulate full slider progress (execute)
    ipc.handle_request(&format!("update_confirmation_progress:{};1.0", conf_id));
    
    // 5. Verify executed and cleared
    sleep(Duration::from_millis(500)).await;
    {
        let state_lock = state.lock().unwrap();
        assert!(state_lock.pending_confirmation.is_none(), "Confirmation should be cleared");
        let hub = &state_lock.sectors[0].hubs[0];
        // The command itself will be echoed by the shell if we wait a bit
        assert!(hub.terminal_output.iter().any(|l| l.text.contains("rm -rf /")));
    }
}
