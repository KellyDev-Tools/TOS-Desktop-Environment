use tos_common::brain::module_manager::ModuleManager;
use tos_common::brain::cortex_registry::CortexRegistry;
use tos_common::services::ai::AiService;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_curator_mcp_context_aggregation() {
    let dir = tempdir().unwrap();
    let modules_dir = dir.path().to_path_buf();

    // Create a curator module manifest
    let curator_dir = modules_dir.join("gitnexus");
    std::fs::create_dir_all(&curator_dir).unwrap();
    
    // The script is in tests/mock_gitnexus.py relative to the workspace root
    let workspace_root = std::env::current_dir().unwrap();
    let mock_script = if workspace_root.ends_with("tests") {
        workspace_root.parent().unwrap().join("tests/mock_gitnexus.py")
    } else {
        workspace_root.join("tests/mock_gitnexus.py")
    };
    
    std::fs::write(curator_dir.join("module.toml"), format!(r#"
id = "gitnexus"
name = "GitNexus"
version = "1.0.0"
module_type = "curator"
author = "TOS"

[connection]
transport = "mcp"

[mcp]
command = "python3"
args = ["{}"]
"#, mock_script.display())).unwrap();

    let manager = Arc::new(ModuleManager::new(modules_dir));
    let registry = Arc::new(Mutex::new(CortexRegistry::new(manager.clone())));
    
    let ai_service = AiService::new();
    ai_service.set_cortex_registry(registry.clone());

    let lock = registry.lock().unwrap();
    let curator = lock.get_curator("gitnexus").expect("Failed to load curator");
    
    let context = curator.get_context("check status", &HashMap::new()).expect("Failed to get context");
    
    assert!(!context.is_empty());
    assert!(context[0].contains("GIT STATUS"));
    assert!(context[0].contains("check status"));
    
    println!("Curator Context: {:?}", context);
}
