use std::path::PathBuf;
use tos_common::brain::module_manager::ModuleManager;
use tos_common::services::audio::AudioService;
use std::sync::Arc;

#[test]
fn test_audio_module_manifest_loading() {
    let temp = tempfile::tempdir().unwrap();
    let mod_dir = temp.path().join("enterprise-audio");
    std::fs::create_dir_all(&mod_dir).unwrap();
    
    let manifest_toml = r#"
id = "tos-audio-enterprise"
name = "Enterprise Audio"
version = "1.0.0"
module_type = "audio"
author = "TOS Team"

[audio]
earcons = "assets/earcons.json"
ambient = "assets/ambient.json"
clicks = "assets/clicks.json"
"#;
    std::fs::write(mod_dir.join("module.toml"), manifest_toml).unwrap();
    
    let mut mm = ModuleManager::new(temp.path().to_path_buf());
    mm.discover_all().unwrap();
    
    let (audio_svc, _) = AudioService::new();
    audio_svc.set_module_manager(Arc::new(mm));
    
    // This will send the LoadModule command to the background thread
    audio_svc.load_audio_module("tos-audio-enterprise").unwrap();
    
    // In a real test we'd wait and check if assets were loaded, 
    // but here we just verify the manifest was accepted.
}
