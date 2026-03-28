use std::sync::Arc;
use tos_common::state::*;
use tos_lib::brain::ipc_handler::IpcHandler;
use std::fs;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_session_persistence_lifecycle() -> anyhow::Result<()> {
    // 1. Setup Brain with Mock Components
    let state = Arc::new(std::sync::Mutex::new(TosState::default()));
    let config = tos_lib::config::TosConfig::default();
    
    // Ensure we are testing the LOCAL fallback first, then we'll verify the daemon logic
    let mut config_local = config.clone();
    config_local.local.persistence = true;
    
    let services = Arc::new(tos_lib::services::ServiceManager::with_config(&config_local));
    let modules = Arc::new(tos_lib::brain::module_manager::ModuleManager::new(PathBuf::from("./dev/fixtures")));
    
    let sid = state.lock().unwrap().sectors[0].id;
    let hid = state.lock().unwrap().sectors[0].hubs[0].id;
    let shell_api = Arc::new(std::sync::Mutex::new(
        tos_lib::brain::shell::ShellApi::new(
            state.clone(), 
            modules, 
            services.ai.clone(), 
            services.heuristic.clone(), 
            sid, hid
        ).unwrap()
    ));

    let handler = IpcHandler::new(state.clone(), shell_api, services.clone());
    let sessions_dir = services.session.sessions_dir();
    let live_file = sessions_dir.join("_live.tos-session");

    // Cleanup from previous runs
    if live_file.exists() {
        fs::remove_file(&live_file)?;
    }

    // 2. Modify state and trigger session write
    {
        let mut st = state.lock().unwrap();
        st.sectors[0].hubs[0].prompt = "test-session-persistence".to_string();
    }

    let response = handler.handle_request("session_live_write:");
    assert_eq!(response, "SESSION_LIVE_WRITTEN");

    // 3. Verify file exists and contains the modification
    assert!(live_file.exists(), "Live session file should be created");
    let content = fs::read_to_string(&live_file)?;
    let saved_state: TosState = serde_json::from_str(&content)?;
    assert_eq!(saved_state.sectors[0].hubs[0].prompt, "test-session-persistence");

    // 4. Test Atomic Rename Success (§5.4.1)
    // The previous write already used the atomic rename pattern (tmp -> live).
    // We can verify this doesn't leave garbage .tmp files.
    let tmp_file = sessions_dir.join("_live.tos-session.tmp");
    assert!(!tmp_file.exists(), "Temp file should be cleaned up after atomic rename");

    // 5. Test Named Session Persistence (§5.4.2)
    let save_res = handler.handle_request("session_save:PRIMARY;test-save");
    assert_eq!(save_res, "SESSION_SAVED: test-save");
    
    let named_file = sessions_dir.join("PRIMARY_test-save.tos-session");
    assert!(named_file.exists(), "Named session file should exist");

    // 6. Test Session Listing (§5.4.3)
    let list_res = handler.handle_request("session_list:PRIMARY");
    assert!(list_res.contains("test-save"), "Listing should include the named session");

    // Cleanup
    fs::remove_file(&live_file)?;
    fs::remove_file(&named_file)?;

    Ok(())
}
