use std::path::PathBuf;
use tos_common::brain::module_manager::ModuleManager;
use tos_common::services::lsp::LspService;
use std::sync::Arc;

#[test]
fn test_language_module_manifest_loading() {
    let temp = tempfile::tempdir().unwrap();
    let mod_dir = temp.path().join("gleam-lang");
    std::fs::create_dir_all(&mod_dir).unwrap();
    
    let manifest_toml = r#"
id = "gleam"
name = "Gleam Language Support"
version = "1.0.0"
module_type = "language"
author = "Community"
file_extensions = [".gleam"]
treesitter_grammar = "bin/gleam.so"

[lsp]
command = "gleam"
args = ["lsp", "--stdio"]
"#;
    std::fs::write(mod_dir.join("module.toml"), manifest_toml).unwrap();
    
    let mut mm = ModuleManager::new(temp.path().to_path_buf());
    mm.discover_all().unwrap();
    
    let modules = mm.list_language_modules();
    assert_eq!(modules.len(), 1);
    let gleam = modules[0];
    assert_eq!(gleam.id, "gleam");
    assert_eq!(gleam.file_extensions.as_ref().unwrap()[0], ".gleam");
    assert_eq!(gleam.lsp.as_ref().unwrap().command, "gleam");
}

#[test]
fn test_lsp_service_dynamic_resolution() {
    let temp = tempfile::tempdir().unwrap();
    let mod_dir = temp.path().join("margo-lang");
    std::fs::create_dir_all(&mod_dir).unwrap();
    
    let manifest_toml = r#"
id = "margo"
name = "Margo Language Support"
version = "0.1.0"
module_type = "language"
author = "TOS Team"
file_extensions = [".margo"]

[lsp]
command = "margo-lsp"
args = ["--serve"]
"#;
    std::fs::write(mod_dir.join("module.toml"), manifest_toml).unwrap();
    
    let mut mm = ModuleManager::new(temp.path().to_path_buf());
    mm.discover_all().unwrap();
    
    let lsp = LspService::new();
    lsp.set_module_manager(Arc::new(mm));
    
    // We can't easily test the spawn without the binary, 
    // but we can verify that start_client doesn't return None immediately for unknown languages 
    // if a module exists.
    // However, it will still return None because spawn() fails.
    
    // To properly test this, we'd need to mock the Command execution or 
    // export the resolution logic to a testable method.
}
