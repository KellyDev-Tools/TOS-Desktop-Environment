use std::path::PathBuf;
use tos_common::brain::module_manager::ModuleManager;
use tos_common::services::bezel::BezelService;
use tos_common::state::TosState;
use std::sync::Arc;

#[test]
fn test_bezel_module_loading_and_update() {
    let temp = tempfile::tempdir().unwrap();
    let mod_dir = temp.path().join("clock-bezel");
    std::fs::create_dir_all(&mod_dir).unwrap();
    
    let manifest_toml = r#"
id = "tos-bezel-clock"
name = "Tactical Clock"
version = "1.0.0"
module_type = "bezel"
author = "TOS Team"

[bezel]
preferred_slot = "right"
"#;
    std::fs::write(mod_dir.join("module.toml"), manifest_toml).unwrap();
    
    let mut mm = ModuleManager::new(temp.path().to_path_buf());
    mm.discover_all().unwrap();
    
    let bezel_svc = BezelService::new();
    bezel_svc.set_module_manager(Arc::new(mm));
    
    bezel_svc.activate_component("tos-bezel-clock").unwrap();
    
    let mut state = TosState::default();
    bezel_svc.update_state(&mut state);
    
    assert_eq!(state.active_bezel_components.len(), 1);
    assert_eq!(state.active_bezel_components[0].id, "tos-bezel-clock");
    assert!(state.active_bezel_components[0].html.contains("Tactical Clock"));
}
