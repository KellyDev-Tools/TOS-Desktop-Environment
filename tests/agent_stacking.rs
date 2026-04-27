use tos_common::state::TosState;
use tos_common::brain::module_manager::ModuleManager;
use tos_common::brain::cortex_registry::CortexRegistry;
use tos_common::services::ai::AiService;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_agent_stacking_prompt_assembly() {
    let dir = tempdir().unwrap();
    let modules_dir = dir.path().to_path_buf();

    // Create a mock agent module manifest
    let agent1_dir = modules_dir.join("agent1");
    std::fs::create_dir_all(&agent1_dir).unwrap();
    std::fs::write(agent1_dir.join("module.toml"), r#"
id = "agent1"
name = "Agent One"
version = "1.0.0"
module_type = "agent"
author = "TOS"

[prompt]
identity = "You are Agent One."
constraints = ["Constraint 1A", "Constraint 1B"]
efficiency = "Efficiency 1"
"#).unwrap();

    let agent2_dir = modules_dir.join("agent2");
    std::fs::create_dir_all(&agent2_dir).unwrap();
    std::fs::write(agent2_dir.join("module.toml"), r#"
id = "agent2"
name = "Agent Two"
version = "1.0.0"
module_type = "agent"
author = "TOS"

[prompt]
identity = "You are Agent Two."
constraints = ["Constraint 2C"]
efficiency = "Efficiency 2"
"#).unwrap();

    let manager = Arc::new(ModuleManager::new(modules_dir));
    let registry = Arc::new(Mutex::new(CortexRegistry::new(manager.clone())));
    
    let ai_service = AiService::new();
    ai_service.set_cortex_registry(registry.clone());

    let mut state = TosState::default();
    state.active_agent_stack = vec!["agent1".to_string(), "agent2".to_string()];

    let prompt = ai_service.assemble_stacked_prompt(&state);

    assert!(prompt.contains("IDENTITY:"));
    assert!(prompt.contains("You are Agent One."));
    assert!(prompt.contains("You are Agent Two."));
    assert!(prompt.contains("CONSTRAINTS:"));
    assert!(prompt.contains("- Constraint 1A"));
    assert!(prompt.contains("- Constraint 1B"));
    assert!(prompt.contains("- Constraint 2C"));
    assert!(prompt.contains("EFFICIENCY:"));
    assert!(prompt.contains("Efficiency 1"));
    assert!(prompt.contains("Efficiency 2"));

    println!("Assembled Prompt:\n{}", prompt);
}
