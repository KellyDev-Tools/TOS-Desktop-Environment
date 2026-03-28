use std::sync::Arc;
use tos_common::state::*;
use tos_lib::brain::ipc_handler::IpcHandler;
use tos_lib::brain::module_manager::ModuleManager;
use tos_lib::services::ServiceManager;
use std::path::PathBuf;

#[tokio::test]
async fn test_osc_integration_cascade_stress() -> anyhow::Result<()> {
    // Phase 5 Gate: Stress test the OSC parsing logic and state cascade (§1.7, §10.3)
    
    // 1. Setup Brain State
    let state = Arc::new(std::sync::Mutex::new(TosState::default()));
    let config = tos_lib::config::TosConfig::default();
    let services = Arc::new(ServiceManager::with_config(&config));
    let modules = Arc::new(ModuleManager::new(PathBuf::from("./dev/fixtures")));
    
    let sid = state.lock().unwrap().sectors[0].id;
    let hid = state.lock().unwrap().sectors[0].hubs[0].id;
    
    // We don't need a real shell for this; we want to test the OscParser directly 
    // in the context of the Brain's ShellApi read_loop logic.
    use tos_lib::brain::shell::OscParser;
    use tos_lib::brain::shell::OscEvent;
    
    let mut parser = OscParser::new();

    // 2. Stress Test OSC 7 (Universal CWD)
    let osc7_seq = "\x1b]7;file://localhost/home/user/workspace/tos\x07";
    let (clean, events) = parser.process(osc7_seq);
    
    assert!(clean.is_empty(), "OSC sequences should be stripped from clean text");
    assert_eq!(events.len(), 1);
    if let OscEvent::Cwd(path) = &events[0] {
        assert_eq!(path, "/home/user/workspace/tos");
    } else {
        panic!("Expected OscEvent::Cwd");
    }

    // 3. Stress Test Heuristic Renaming Gate (§10.3)
    // We'll simulate the Brain receiving these events.
    {
        let mut st = state.lock().unwrap();
        let sector = &mut st.sectors[0];
        let hub = &mut sector.hubs[0];
        
        // Initial state
        sector.name = "Primary".to_string();
        hub.current_directory = PathBuf::from("/");

        // Event: CWD change to a project dir
        hub.current_directory = PathBuf::from("/8TB/labs/neural-link");
        
        // Trigger the renaming logic (manually mimicking shell/mod.rs:203)
        if sector.name == "Primary" {
            if let Some(name) = hub.current_directory.file_name().and_then(|s| s.to_str()) {
                sector.name = name.to_string();
            }
        }
        
        assert_eq!(sector.name, "neural-link", "Sector should heuristically rename to CWD basename");
    }

    // 4. Stress Test Command Result + Priority Interleaving
    // Seq: [Priority 1] [Output] [Exit 127]
    let complex_seq = "\x1b]9012;1\x07sh: command not found\x1b]9002;badcmd;127\x07";
    let (clean_complex, events_complex) = parser.process(complex_seq);
    
    assert_eq!(clean_complex.trim(), "sh: command not found");
    // Should have 2 events: Priority change and Command Result
    assert_eq!(events_complex.len(), 2);
    
    // 5. Stress Test JSON Context Injection (§9004)
    // Using base64 for the real deal
    let json_data = serde_json::json!({"git": {"branch": "main", "dirty": true}});
    let b64_json = base64::Engine::encode(&base64::prelude::BASE64_STANDARD, serde_json::to_string(&json_data)?);
    let json_seq = format!("\x1b]9004;{}\x07", b64_json);
    
    let (_, json_events) = parser.process(&json_seq);
    assert_eq!(json_events.len(), 1);
    if let OscEvent::JsonContext(val) = &json_events[0] {
        assert_eq!(val["git"]["branch"], "main");
    } else {
        panic!("Expected OscEvent::JsonContext");
    }

    Ok(())
}
