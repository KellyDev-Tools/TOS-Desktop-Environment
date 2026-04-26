use std::collections::HashMap;
use std::path::PathBuf;
use tos_common::modules::AiQuery;
use tos_common::brain::module_manager::ModuleManager;

#[tokio::test(flavor = "multi_thread")]
async fn test_assistant_load_with_connection() {
    let temp = tempfile::tempdir().unwrap();
    let module_dir = temp.path().join("test-assistant");
    std::fs::create_dir_all(&module_dir).unwrap();
    
    let manifest_toml = r#"
id = "test-assistant"
name = "Test Assistant"
version = "0.1.0"
module_type = "assistant"
author = "Tester"

[connection]
transport = "http"
endpoint = "http://localhost:12345"
"#;
    std::fs::write(module_dir.join("module.toml"), manifest_toml).unwrap();
    
    let manager = ModuleManager::new(temp.path().to_path_buf());
    let assistant = manager.load_assistant("test-assistant").unwrap();
    
    assert_eq!(assistant.id(), "test-assistant");
    assert_eq!(assistant.name(), "Test Assistant");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_curator_mcp_transport_stub() {
    let temp = tempfile::tempdir().unwrap();
    let module_dir = temp.path().join("test-curator");
    std::fs::create_dir_all(&module_dir).unwrap();
    
    let manifest_toml = r#"
id = "test-curator"
name = "Test Curator"
version = "0.1.0"
module_type = "curator"
author = "Tester"

[connection]
transport = "mcp"
endpoint = "http://localhost:3000/mcp"

[mcp]
command = "echo"
args = ["{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"content\":[\"Hello from MCP\"]}}"]
"#;
    std::fs::write(module_dir.join("module.toml"), manifest_toml).unwrap();
    
    let manager = ModuleManager::new(temp.path().to_path_buf());
    let curator = manager.load_curator("test-curator").unwrap();
    
    let context = curator.get_context("hi", &HashMap::new()).unwrap();
    // Since we used echo with a hardcoded JSON, mcp_stdio_call should read it.
    assert_eq!(context[0], "Hello from MCP");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_stdio_ai_fallback() {
    let temp = tempfile::tempdir().unwrap();
    let module_dir = temp.path().join("test-ai");
    std::fs::create_dir_all(&module_dir).unwrap();
    
    // Create a mock executable that just echoes a valid AiResponse JSON
    let exe_path = if cfg!(windows) {
        module_dir.join("mock_ai.bat")
    } else {
        module_dir.join("mock_ai.sh")
    };
    
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let mut f = std::fs::File::create(&exe_path).unwrap();
        f.write_all(b"#!/bin/bash\necho '{\"id\":\"00000000-0000-0000-0000-000000000000\",\"choice\":{\"role\":\"assistant\",\"content\":\"mock response\"},\"usage\":{\"tokens\":10},\"status\":\"complete\"}'").unwrap();
        f.sync_all().unwrap();
        drop(f);

        let mut perms = std::fs::metadata(&exe_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&exe_path, perms).unwrap();
    }
    
    let manifest_toml = format!(r#"
id = "test-ai"
name = "Test AI"
version = "0.1.0"
module_type = "ai"
author = "Tester"

[executable]
path = "{}"
args = []

[connection]
transport = "stdio"
"#, exe_path.file_name().unwrap().to_str().unwrap());

    std::fs::write(module_dir.join("module.toml"), manifest_toml).unwrap();
    
    let manager = ModuleManager::new(temp.path().to_path_buf());
    let ai = manager.load_ai("test-ai").unwrap();
    
    let query = AiQuery {
        prompt: "hello".to_string(),
        context: vec![],
        stream: false,
        auth: HashMap::new(),
    };
    
    let resp = ai.query(query).unwrap();
    assert_eq!(resp.choice.content, "mock response");
}
