use std::sync::{Arc, Mutex};
use tos_common::brain::ipc_handler::IpcHandler;
use tos_common::platform::RemoteServer;

#[tokio::test]
async fn test_opensearch_xml_retrieval() {
    let state = Arc::new(Mutex::new(tos_common::TosState::default()));
    let services = Arc::new(tos_common::services::ServiceManager::new());
    
    // Create a dummy shell for the IPC handler
    let mm = Arc::new(tos_common::brain::module_manager::ModuleManager::new(std::path::PathBuf::from("/tmp")));
    let shell = Arc::new(Mutex::new(tos_common::brain::shell::ShellApi::new(
        state.clone(),
        mm,
        services.ai.clone(),
        services.heuristic.clone(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    ).unwrap()));

    let ipc = Arc::new(IpcHandler::new(state, shell, services));
    let server = RemoteServer::new(ipc);
    
    let xml = server.generate_opensearch_xml();
    assert!(xml.contains("<OpenSearchDescription"));
    assert!(xml.contains("<ShortName>TOS Brain</ShortName>"));
    assert!(xml.contains("http://localhost:7000/search?q={searchTerms}"));
}
