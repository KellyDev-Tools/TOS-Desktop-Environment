use std::sync::Arc;
use tos_common::state::*;
use tos_common::ipc::*;

#[tokio::test]
async fn test_ai_manual_submit_gate() {
    // Phase 5 Gate Test: Confirm no AI skill can auto-submit a command
    // Staging only, always editable. Features §4.12
    
    // 1. Setup Brain State
    let state = Arc::new(std::sync::Mutex::new(TosState::default()));
    let config = tos_common::config::TosConfig::default();
    let services = Arc::new(tos_common::services::ServiceManager::with_config(&config));
    
    // Simulate an AI suggestion arriving over IPC
    let suggestion_json = serde_json::json!({
        "command": "rm -rf /tmp/test",
        "explanation": "Clear temporary directory"
    });
    
    let payload = suggestion_json.to_string();
    let request = format!("ai_stage_command:{}", payload);
    
    // The modules and shell_api are required for full initialization
    let modules = Arc::new(tos_common::brain::module_manager::ModuleManager::new(std::path::PathBuf::from("./dev/fixtures")));
    let sid = state.lock().unwrap().sectors[0].id;
    let hid = state.lock().unwrap().sectors[0].hubs[0].id;
    let shell_api = Arc::new(std::sync::Mutex::new(tos_common::brain::shell::ShellApi::new(state.clone(), modules, services.ai.clone(), services.heuristic.clone(), sid, hid).unwrap()));
    
    let handler = tos_common::brain::ipc_handler::IpcHandler::new(state.clone(), shell_api, services);
    
    // 2. Stage the command
    let response = handler.dispatch(&request);
    assert_eq!(response, "AI_COMMAND_STAGED");
    
    // 3. Verify it is ONLY staged, NOT submitted
    {
        let final_state = state.lock().unwrap();
        let hub = &final_state.sectors[0].hubs[0];
        
        // Assert command is staged
        assert_eq!(hub.staged_command, Some("rm -rf /tmp/test".to_string()));
        assert_eq!(hub.ai_explanation, Some("Clear temporary directory".to_string()));
        
        // Assert NOT submitted (prompt is empty, last_command is empty)
        assert_eq!(hub.prompt, "");
        assert!(hub.terminal_output.is_empty());
    }
    
    // 4. Verify the user accepts the command
    let accept_req = "ai_suggestion_accept";
    let accept_res = handler.dispatch(accept_req);
    assert_eq!(accept_res, "AI_SUGGESTION_ACCEPTED");
    
    // 5. Verify the staged command moved into the prompt for editing
    {
        let final_state = state.lock().unwrap();
        let hub = &final_state.sectors[0].hubs[0];
        
        assert_eq!(hub.staged_command, None);
        assert_eq!(hub.prompt, "rm -rf /tmp/test"); 
        // User must still hit ENTER to actually execute it.
    }
}
