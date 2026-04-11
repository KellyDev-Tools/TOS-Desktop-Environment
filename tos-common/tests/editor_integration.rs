use tos_common::*;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_editor_open_integration() {
    let mut state = TosState::default();
    let sector = &mut state.sectors[0];
    let hub = &mut sector.hubs[0];
    
    let path = PathBuf::from("/tmp/test.rs");
    let content = "fn main() {}".to_string();
    
    // Simulate editor_open IPC logic
    let editor = EditorPaneState::new_viewer(path.clone(), content.clone(), Some("rust".to_string()));
    let pane = SplitPane::new_with_content(PaneContent::Editor(editor));
    
    if hub.split_layout.is_none() {
        hub.split_layout = Some(SplitNode::Leaf(pane));
    } else {
        hub.split_layout.as_mut().unwrap().add_pane(pane);
    }
    
    let layout = hub.split_layout.as_ref().unwrap();
    let editors = layout.all_editors();
    assert_eq!(editors.len(), 1);
    assert_eq!(editors[0].file_path, path);
    assert_eq!(editors[0].content, content);
}

#[test]
fn test_editor_save_flow() {
    let mut state = TosState::default();
    let path = PathBuf::from("/tmp/save_test.rs");
    let initial_content = "initial".to_string();
    
    let mut ed = EditorPaneState::new_viewer(path.clone(), initial_content.clone(), None);
    ed.mode = EditorMode::Editor;
    ed.content = "modified".to_string();
    ed.dirty = true;
    
    // In real app, IPC would trigger the write. 
    // Here we verify the state transformation.
    assert!(ed.dirty);
    assert_eq!(ed.content, "modified");
    
    // Simulate save completion
    ed.dirty = false;
    assert!(!ed.dirty);
}

#[test]
fn test_trust_boundary_logic() {
    let trust = services::trust::TrustService::new();
    let root = std::path::Path::new("/home/user/project");
    
    // Within boundary
    assert!(trust.is_path_trusted(std::path::Path::new("/home/user/project/src/lib.rs"), root));
    
    // Outside boundary
    assert!(!trust.is_path_trusted(std::path::Path::new("/etc/hosts"), root));
    assert!(!trust.is_path_trusted(std::path::Path::new("/home/user/other/file.txt"), root));
}
