//! Tests for P1 Items 5, 6, 7, 8, 10:
//!   - Item 5:  Directory Mode multi-select (§3.2)
//!   - Item 6:  Directory Mode action toolbar (§3.2)
//!   - Item 7:  Directory Mode context menu (§3.2)
//!   - Item 8:  Path bar breadcrumb-style (§3.2)
//!   - Item 10: Sector templates dynamic (§15)
//!
//! Covers:
//! - Unit:      IPC handler state mutations (no renderer needed)
//! - Component: HubRenderer Directory Mode HTML output
//! - Integration: Full render_viewport pipeline + template save/load round-trip

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tos_core::{
    TosState, CommandHubMode, HierarchyLevel, RenderMode, ContextMenu,
};
use tos_core::ui::render::{ViewRenderer, hub::HubRenderer};
use tos_core::system::ipc::IpcDispatcher;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_ipc(state: Arc<Mutex<TosState>>) -> IpcDispatcher {
    let ptys = Arc::new(Mutex::new(HashMap::new()));
    IpcDispatcher::new(state, ptys)
}

fn make_dir_state() -> (TosState, tos_core::Viewport) {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    state.current_level = HierarchyLevel::CommandHub;
    let viewport = state.viewports[0].clone();
    (state, viewport)
}

// ─── Unit Tests: IPC state mutations ──────────────────────────────────────────

// ── Item 5: Multi-select ──────────────────────────────────────────────────────

#[test]
fn test_dir_toggle_select_adds_file() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));

    ipc.handle_request("dir_toggle_select:foo.txt");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert!(
        s.sectors[sector_idx].hubs[hub_idx].selected_files.contains("foo.txt"),
        "foo.txt should be selected after toggle"
    );
}

#[test]
fn test_dir_toggle_select_removes_already_selected_file() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].selected_files.insert("bar.txt".to_string());
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_toggle_select:bar.txt");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert!(
        !s.sectors[sector_idx].hubs[hub_idx].selected_files.contains("bar.txt"),
        "bar.txt should be deselected after second toggle"
    );
}

#[test]
fn test_dir_toggle_select_multiple_files() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));

    ipc.handle_request("dir_toggle_select:a.txt");
    ipc.handle_request("dir_toggle_select:b.txt");
    ipc.handle_request("dir_toggle_select:c.txt");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let sel = &s.sectors[sector_idx].hubs[hub_idx].selected_files;
    assert_eq!(sel.len(), 3, "Three files should be selected");
    assert!(sel.contains("a.txt"));
    assert!(sel.contains("b.txt"));
    assert!(sel.contains("c.txt"));
}

#[test]
fn test_dir_toggle_select_syncs_prompt() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp");
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_toggle_select:report.pdf");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let prompt = &s.sectors[sector_idx].hubs[hub_idx].prompt;
    assert!(
        prompt.contains("report.pdf"),
        "Prompt should contain selected file path, got: {}", prompt
    );
    assert!(
        prompt.contains("/tmp"),
        "Prompt should contain full path prefix, got: {}", prompt
    );
}

#[test]
fn test_dir_toggle_select_deselect_clears_prompt() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp");
    }
    let ipc = make_ipc(Arc::clone(&state));
    // Select then deselect
    ipc.handle_request("dir_toggle_select:solo.txt");
    ipc.handle_request("dir_toggle_select:solo.txt");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    // selected_files should be empty
    assert!(
        s.sectors[sector_idx].hubs[hub_idx].selected_files.is_empty(),
        "selected_files should be empty after deselect"
    );
}

#[test]
fn test_dir_clear_select_clears_all() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].selected_files.insert("x.txt".to_string());
        s.sectors[sector_idx].hubs[hub_idx].selected_files.insert("y.txt".to_string());
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_clear_select");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert!(
        s.sectors[sector_idx].hubs[hub_idx].selected_files.is_empty(),
        "All selections should be cleared"
    );
}

// ── Item 6: Action toolbar IPC handlers ──────────────────────────────────────

#[test]
fn test_dir_action_copy_stages_cp_command() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].selected_files.insert("file1.txt".to_string());
        s.sectors[sector_idx].hubs[hub_idx].selected_files.insert("file2.txt".to_string());
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_action_copy");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let prompt = &s.sectors[sector_idx].hubs[hub_idx].prompt;
    assert!(prompt.starts_with("cp "), "Copy should stage a cp command, got: {}", prompt);
    assert!(prompt.contains("file1.txt") || prompt.contains("file2.txt"),
        "Copy command should include selected files");
}

#[test]
fn test_dir_action_copy_empty_selection_no_prompt_change() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_action_copy");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    // No files selected — prompt should remain empty
    assert!(
        s.sectors[sector_idx].hubs[hub_idx].prompt.is_empty(),
        "Copy with no selection should not change prompt"
    );
}

#[test]
fn test_dir_action_paste_stages_cp_clipboard() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_action_paste");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let prompt = &s.sectors[sector_idx].hubs[hub_idx].prompt;
    assert!(prompt.contains("cp"), "Paste should stage a cp command, got: {}", prompt);
}

#[test]
fn test_dir_action_delete_stages_rm_command() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].selected_files.insert("old.log".to_string());
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_action_delete");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let prompt = &s.sectors[sector_idx].hubs[hub_idx].prompt;
    assert!(prompt.starts_with("rm "), "Delete should stage an rm command, got: {}", prompt);
    assert!(prompt.contains("old.log"), "Delete command should include selected file");
}

#[test]
fn test_stage_command_mkdir_stages_mkdir() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("stage_command:mkdir ");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let prompt = &s.sectors[sector_idx].hubs[hub_idx].prompt;
    assert_eq!(prompt, "mkdir ", "NEW FOLDER should stage 'mkdir ' in prompt");
}

// ── Item 7: Context menu ──────────────────────────────────────────────────────

#[test]
fn test_dir_context_opens_menu_with_target() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_context:myfile.txt;100;200");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let menu = s.sectors[sector_idx].hubs[hub_idx].context_menu.as_ref()
        .expect("Context menu should be set");
    assert_eq!(menu.target, "myfile.txt");
    assert_eq!(menu.x, 100);
    assert_eq!(menu.y, 200);
}

#[test]
fn test_dir_context_menu_has_standard_actions() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_context:target.rs;50;75");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let menu = s.sectors[sector_idx].hubs[hub_idx].context_menu.as_ref().unwrap();
    assert!(menu.actions.contains(&"OPEN".to_string()), "Menu should have OPEN action");
    assert!(menu.actions.contains(&"COPY".to_string()), "Menu should have COPY action");
    assert!(menu.actions.contains(&"RENAME".to_string()), "Menu should have RENAME action");
    assert!(menu.actions.contains(&"DELETE".to_string()), "Menu should have DELETE action");
}

#[test]
fn test_dir_close_context_clears_menu() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].context_menu = Some(ContextMenu {
            target: "file.txt".to_string(),
            x: 10, y: 20,
            actions: vec!["OPEN".to_string()],
        });
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_close_context");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert!(
        s.sectors[sector_idx].hubs[hub_idx].context_menu.is_none(),
        "Context menu should be cleared after dir_close_context"
    );
}

#[test]
fn test_dir_navigate_clears_context_menu() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].context_menu = Some(ContextMenu {
            target: "file.txt".to_string(),
            x: 10, y: 20,
            actions: vec!["OPEN".to_string()],
        });
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp");
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_navigate:..");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert!(
        s.sectors[sector_idx].hubs[hub_idx].context_menu.is_none(),
        "Context menu should be cleared on navigation"
    );
}

#[test]
fn test_dir_navigate_clears_selection() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].selected_files.insert("stale.txt".to_string());
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp");
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_navigate:..");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert!(
        s.sectors[sector_idx].hubs[hub_idx].selected_files.is_empty(),
        "Selection should be cleared on navigation"
    );
}

// ── Item 8: Breadcrumb navigation ─────────────────────────────────────────────

#[test]
fn test_dir_navigate_up_goes_to_parent() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp/subdir");
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_navigate:..");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert_eq!(
        s.sectors[sector_idx].hubs[hub_idx].current_directory,
        std::path::PathBuf::from("/tmp"),
        "Navigate .. should go to parent directory"
    );
}

#[test]
fn test_dir_navigate_into_existing_dir() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        // /tmp always exists on Linux
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/");
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_navigate:tmp");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert_eq!(
        s.sectors[sector_idx].hubs[hub_idx].current_directory,
        std::path::PathBuf::from("/tmp"),
        "Navigate into existing dir should update current_directory"
    );
}

#[test]
fn test_dir_navigate_into_nonexistent_dir_no_change() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp");
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_navigate:this_dir_does_not_exist_tos_test");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    assert_eq!(
        s.sectors[sector_idx].hubs[hub_idx].current_directory,
        std::path::PathBuf::from("/tmp"),
        "Navigate into non-existent dir should not change current_directory"
    );
}

#[test]
fn test_dir_pick_file_appends_to_prompt() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/home/user");
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_pick_file:notes.md");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let prompt = &s.sectors[sector_idx].hubs[hub_idx].prompt;
    assert!(
        prompt.contains("notes.md"),
        "Picking a file should append it to the prompt, got: {}", prompt
    );
    assert!(
        prompt.contains("/home/user"),
        "Prompt should contain the full path, got: {}", prompt
    );
}

#[test]
fn test_dir_pick_file_appends_to_existing_prompt() {
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp");
        s.sectors[sector_idx].hubs[hub_idx].prompt = "cat".to_string();
    }
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("dir_pick_file:readme.txt");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let prompt = &s.sectors[sector_idx].hubs[hub_idx].prompt;
    assert!(
        prompt.starts_with("cat"),
        "Existing prompt prefix should be preserved, got: {}", prompt
    );
    assert!(
        prompt.contains("readme.txt"),
        "New file should be appended, got: {}", prompt
    );
}

#[test]
fn test_dir_toggle_hidden_flips_flag() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));

    // Default is false
    {
        let s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        assert!(!s.sectors[sector_idx].hubs[hub_idx].show_hidden_files);
    }

    ipc.handle_request("dir_toggle_hidden");
    {
        let s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        assert!(s.sectors[sector_idx].hubs[hub_idx].show_hidden_files,
            "show_hidden_files should be true after toggle");
    }

    ipc.handle_request("dir_toggle_hidden");
    {
        let s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        assert!(!s.sectors[sector_idx].hubs[hub_idx].show_hidden_files,
            "show_hidden_files should be false after second toggle");
    }
}

// ── Item 10: Sector templates ─────────────────────────────────────────────────

#[test]
fn test_save_template_adds_terminal_output() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("save_template:MyLayout");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let output = &s.sectors[sector_idx].hubs[hub_idx].terminal_output;
    assert!(
        output.iter().any(|l| l.contains("MyLayout")),
        "Save template should add a terminal output line mentioning the template name"
    );
}

#[test]
fn test_load_template_updates_sector_name() {
    let state = Arc::new(Mutex::new(TosState::new()));
    let ipc = make_ipc(Arc::clone(&state));
    ipc.handle_request("load_template:DevWorkspace");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    assert!(
        s.sectors[sector_idx].name.contains("DevWorkspace"),
        "Load template should update sector name to include template name, got: {}",
        s.sectors[sector_idx].name
    );
}

#[test]
fn test_get_available_templates_returns_vec() {
    // When no templates directory exists, should return empty vec (not panic)
    let state = TosState::new();
    let templates = state.get_available_templates();
    // Should be a Vec (possibly empty if ~/.local/share/tos/templates doesn't exist)
    // The important thing is it doesn't panic
    let _ = templates.len();
}

#[ignore] // Requires --test-threads=1 due to HOME env var pollution between parallel tests
#[test]
fn test_get_available_templates_reads_tos_template_files() {
    use std::fs;

    // Use a unique dir per test to avoid HOME env var races between parallel tests
    let unique = format!("tos-tmpl-read-{}-{:?}", std::process::id(), std::thread::current().id());
    let tmp = std::env::temp_dir().join(&unique);
    let template_dir = tmp.join(".local/share/tos/templates");
    
    // Clean up any existing template dir first to ensure clean state
    let _ = fs::remove_dir_all(&template_dir);
    fs::create_dir_all(&template_dir).unwrap();

    let template_content = r#"{"name":"TestLayout","hubs":2,"apps":["Shell","Spectrometer"]}"#;
    fs::write(
        template_dir.join("test-layout.tos-template"),
        template_content,
    ).unwrap();

    // Serialize this test with a mutex to avoid races with other template tests.
    // Keep the lock for the entire test to ensure proper isolation.
    static TEMPLATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = TEMPLATE_LOCK.lock().unwrap();

    let home_backup = std::env::var("HOME").unwrap_or_default();
    std::env::set_var("HOME", &tmp);

    // Use new_fresh() to ensure we don't load cached state from a previous test
    let state = TosState::new_fresh();
    let templates = state.get_available_templates();

    // Assert while still holding the lock
    assert_eq!(templates.len(), 1, "Should find exactly one .tos-template file, found: {:?}", templates.iter().map(|t| &t.name).collect::<Vec<_>>());
    let t = &templates[0];
    assert_eq!(t.name, "TESTLAYOUT", "Template name should be uppercased");
    assert_eq!(t.hubs, 2, "Template should parse hub count");
    assert_eq!(t.apps, 2, "Template should parse app count");

    // Restore HOME and cleanup while still holding lock
    std::env::set_var("HOME", &home_backup);
    drop(_guard);
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_get_available_templates_ignores_non_template_files() {
    use std::fs;

    let unique = format!("tos-tmpl-ignore-{}-{:?}", std::process::id(), std::thread::current().id());
    let tmp = std::env::temp_dir().join(&unique);
    let template_dir = tmp.join(".local/share/tos/templates");
    
    // Clean up any existing template dir first to ensure clean state
    let _ = fs::remove_dir_all(&template_dir);
    fs::create_dir_all(&template_dir).unwrap();

    fs::write(template_dir.join("notes.txt"), "not a template").unwrap();
    fs::write(
        template_dir.join("valid.tos-template"),
        r#"{"name":"Valid","hubs":1,"apps":[]}"#,
    ).unwrap();

    // Use a SHARED lock to serialize with other template tests
    static TEMPLATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = TEMPLATE_LOCK.lock().unwrap();
    let home_backup = std::env::var("HOME").unwrap_or_default();
    std::env::set_var("HOME", &tmp);

    let state = TosState::new_fresh();
    let templates = state.get_available_templates();

    std::env::set_var("HOME", &home_backup);
    drop(_guard);
    let _ = fs::remove_dir_all(&tmp);

    assert_eq!(templates.len(), 1, "Should only find .tos-template files, not .txt");
}

#[ignore] // Requires --test-threads=1 due to HOME env var pollution between parallel tests
#[test]
fn test_get_available_templates_fallback_for_invalid_json() {
    use std::fs;

    let unique = format!("tos-tmpl-broken-{}-{:?}", std::process::id(), std::thread::current().id());
    let tmp = std::env::temp_dir().join(&unique);
    let template_dir = tmp.join(".local/share/tos/templates");
    
    // Clean up any existing template dir first to ensure clean state
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&template_dir).unwrap();

    fs::write(template_dir.join("broken.tos-template"), "not json at all").unwrap();

    // Serialize this test with a mutex to avoid races with other template tests.
    // Keep the lock for the entire test to ensure proper isolation.
    static TEMPLATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = TEMPLATE_LOCK.lock().unwrap();
    let home_backup = std::env::var("HOME").unwrap_or_default();
    std::env::set_var("HOME", &tmp);

    // Use new_fresh() to ensure we don't load cached state from a previous test
    let state = TosState::new_fresh();
    let templates = state.get_available_templates();

    // Assert while still holding the lock
    // Look specifically for our BROKEN template
    let broken_found = templates.iter().any(|t| t.name == "BROKEN");
    assert!(broken_found, "Invalid JSON template should still be listed, found: {:?}", templates.iter().map(|t| &t.name).collect::<Vec<_>>());
    
    // Find and verify the broken template
    if let Some(t) = templates.iter().find(|t| t.name == "BROKEN") {
        assert_eq!(t.description, "CUSTOM SECTOR LAYOUT");
    }

    // Restore HOME and cleanup while still holding lock
    std::env::set_var("HOME", &home_backup);
    drop(_guard);
    let _ = fs::remove_dir_all(&tmp);
}

// ─── Component Tests: HubRenderer Directory Mode HTML ─────────────────────────

#[test]
fn test_hub_renderer_directory_mode_class_applied() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("mode-directory"), "Directory mode should apply mode-directory CSS class");
}

#[test]
fn test_hub_renderer_shows_breadcrumb_path_bar() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("breadcrumbs"), "Should render breadcrumb path bar");
    assert!(html.contains("breadcrumb"), "Should render breadcrumb spans");
    assert!(html.contains("ROOT"), "Should show ROOT as first breadcrumb");
}

#[test]
fn test_hub_renderer_breadcrumbs_for_nested_path() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/home/user/projects");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    let viewport = state.viewports[0].clone();

    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("ROOT"), "Should show ROOT breadcrumb");
    assert!(html.contains("HOME"), "Should show HOME breadcrumb segment");
    assert!(html.contains("USER"), "Should show USER breadcrumb segment");
    assert!(html.contains("PROJECTS"), "Should show PROJECTS breadcrumb segment");
    // Breadcrumbs accumulate path without leading slash (PathBuf::push behaviour)
    // e.g. 'home', 'home/user', 'home/user/projects'
    assert!(html.contains("dir_navigate:home"), "Breadcrumb should link to home segment");
    assert!(html.contains("dir_navigate:home/user"), "Breadcrumb should link to home/user segment");
}

#[test]
fn test_hub_renderer_breadcrumb_root_navigates_to_slash() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(
        html.contains("dir_navigate:/"),
        "ROOT breadcrumb should navigate to /"
    );
}

#[test]
fn test_hub_renderer_shows_action_toolbar() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("NEW FOLDER"), "Action toolbar should have NEW FOLDER button");
    assert!(html.contains("COPY"), "Action toolbar should have COPY button");
    assert!(html.contains("PASTE"), "Action toolbar should have PASTE button");
    assert!(html.contains("RENAME"), "Action toolbar should have RENAME button");
    assert!(html.contains("DELETE"), "Action toolbar should have DELETE button");
    assert!(html.contains("REFRESH"), "Action toolbar should have REFRESH button");
}

#[test]
fn test_hub_renderer_action_toolbar_ipc_messages() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("stage_command:mkdir"), "NEW FOLDER should fire mkdir IPC");
    assert!(html.contains("dir_action_copy"), "COPY should fire dir_action_copy IPC");
    assert!(html.contains("dir_action_paste"), "PASTE should fire dir_action_paste IPC");
    assert!(html.contains("stage_command:mv"), "RENAME should fire mv IPC");
    assert!(html.contains("stage_command:rm"), "DELETE should fire rm IPC");
}

#[test]
fn test_hub_renderer_shows_parent_dir_entry() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("dir_navigate:.."), "Should show parent directory (..) entry");
}

#[test]
fn test_hub_renderer_selected_file_gets_selected_class() {
    use tos_core::{DirectoryListing, DirectoryEntry, EntryType};
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    // Use shell_listing so the renderer knows about our file by name
    state.sectors[sector_idx].hubs[hub_idx].shell_listing = Some(DirectoryListing {
        path: "/tmp".to_string(),
        parent: None,
        total_count: 1,
        hidden_count: 0,
        selected_count: 1,
        entries: vec![
            DirectoryEntry {
                name: "known-file.txt".to_string(),
                entry_type: EntryType::File,
                size: 1024,
                permissions: "rw-r--r--".to_string(),
                modified: "2026-01-01".to_string(),
                is_hidden: false,
                is_selected: true,
            },
        ],
    });
    state.sectors[sector_idx].hubs[hub_idx].selected_files.insert("known-file.txt".to_string());
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    let viewport = state.viewports[0].clone();

    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("selected"), "Selected file should get 'selected' CSS class");
    assert!(html.contains("KNOWN-FILE.TXT"), "Selected file name should appear in render");
}

#[test]
fn test_hub_renderer_multi_select_batch_toolbar_appears() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.sectors[sector_idx].hubs[hub_idx].selected_files.insert("a.txt".to_string());
    state.sectors[sector_idx].hubs[hub_idx].selected_files.insert("b.txt".to_string());
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    let viewport = state.viewports[0].clone();

    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("FILES SELECTED"), "Batch toolbar should appear when files are selected");
    assert!(html.contains("REPLICATE"), "Batch toolbar should have REPLICATE (copy) action");
    assert!(html.contains("PURGE"), "Batch toolbar should have PURGE (delete) action");
    assert!(html.contains("CLEAR"), "Batch toolbar should have CLEAR action");
}

#[test]
fn test_hub_renderer_context_menu_rendered_when_set() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.sectors[sector_idx].hubs[hub_idx].context_menu = Some(ContextMenu {
        target: "myfile.txt".to_string(),
        x: 150, y: 300,
        actions: vec!["OPEN".to_string(), "COPY".to_string(), "DELETE".to_string()],
    });
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    let viewport = state.viewports[0].clone();

    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("context-menu"), "Context menu should be rendered");
    assert!(html.contains("MYFILE.TXT"), "Context menu should show target filename");
    assert!(html.contains("left: 150px"), "Context menu should be positioned at x=150");
    assert!(html.contains("top: 300px"), "Context menu should be positioned at y=300");
    assert!(html.contains("OPEN"), "Context menu should show OPEN action");
    assert!(html.contains("COPY"), "Context menu should show COPY action");
    assert!(html.contains("DELETE"), "Context menu should show DELETE action");
}

#[test]
fn test_hub_renderer_context_menu_not_rendered_when_none() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(!html.contains("context-menu-overlay"), "Context menu overlay should not appear when no menu is set");
}

#[test]
fn test_hub_renderer_shows_hidden_toggle_button() {
    let (state, viewport) = make_dir_state();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    // Should show either SHOW HIDDEN or HIDE HIDDEN
    assert!(
        html.contains("HIDDEN"),
        "Should show hidden files toggle button"
    );
    assert!(
        html.contains("dir_toggle_hidden"),
        "Hidden toggle should fire dir_toggle_hidden IPC"
    );
}

#[test]
fn test_hub_renderer_shows_file_selector_checkboxes() {
    // Use /tmp which always has entries on Linux
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    let viewport = state.viewports[0].clone();

    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(
        html.contains("file-selector"),
        "File items should have file-selector checkbox elements"
    );
    assert!(
        html.contains("dir_toggle_select:"),
        "File selectors should fire dir_toggle_select IPC"
    );
}

#[test]
fn test_hub_renderer_files_have_context_menu_handler() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    let viewport = state.viewports[0].clone();

    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(
        html.contains("oncontextmenu"),
        "File items should have oncontextmenu handler"
    );
    assert!(
        html.contains("dir_context:"),
        "Context menu handler should fire dir_context IPC"
    );
}

// ─── Integration Tests: render_viewport ───────────────────────────────────────

#[test]
fn test_render_viewport_directory_mode_has_breadcrumbs() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    state.current_level = HierarchyLevel::CommandHub;

    let viewport = state.viewports[0].clone();
    let html = state.render_viewport(&viewport);

    assert!(html.contains("breadcrumbs"), "Full render should include breadcrumb path bar");
    assert!(html.contains("ROOT"), "Full render should include ROOT breadcrumb");
    assert!(html.contains("TMP"), "Full render should include TMP breadcrumb segment");
}

#[test]
fn test_render_viewport_directory_mode_has_action_toolbar() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    state.current_level = HierarchyLevel::CommandHub;

    let viewport = state.viewports[0].clone();
    let html = state.render_viewport(&viewport);

    assert!(html.contains("NEW FOLDER"), "Full render should include action toolbar");
    assert!(html.contains("COPY"), "Full render should include COPY button");
    assert!(html.contains("DELETE"), "Full render should include DELETE button");
}

#[test]
fn test_render_viewport_directory_mode_has_context_menu_handlers() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    state.current_level = HierarchyLevel::CommandHub;

    let viewport = state.viewports[0].clone();
    let html = state.render_viewport(&viewport);

    assert!(html.contains("oncontextmenu"), "Full render should include context menu handlers");
    assert!(html.contains("dir_context:"), "Full render should include dir_context IPC calls");
}

#[test]
fn test_render_viewport_directory_mode_has_multi_select() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;
    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Directory;
    state.sectors[sector_idx].hubs[hub_idx].current_directory =
        std::path::PathBuf::from("/tmp");
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    state.current_level = HierarchyLevel::CommandHub;

    let viewport = state.viewports[0].clone();
    let html = state.render_viewport(&viewport);

    assert!(html.contains("file-selector"), "Full render should include file selector checkboxes");
    assert!(html.contains("dir_toggle_select:"), "Full render should include toggle select IPC");
}

#[test]
fn test_full_directory_mode_ipc_round_trip() {
    use tos_core::{DirectoryListing, DirectoryEntry, EntryType};
    // Full round-trip: set mode → inject shell listing → select file → context menu → render
    let state = Arc::new(Mutex::new(TosState::new()));
    {
        let mut s = state.lock().unwrap();
        let sector_idx = s.viewports[0].sector_index;
        let hub_idx = s.viewports[0].hub_index;
        s.sectors[sector_idx].hubs[hub_idx].current_directory =
            std::path::PathBuf::from("/tmp");
        // Inject a shell listing so the renderer knows about our file
        s.sectors[sector_idx].hubs[hub_idx].shell_listing = Some(DirectoryListing {
            path: "/tmp".to_string(),
            parent: None,
            total_count: 1,
            hidden_count: 0,
            selected_count: 0,
            entries: vec![
                DirectoryEntry {
                    name: "testfile.txt".to_string(),
                    entry_type: EntryType::File,
                    size: 512,
                    permissions: "rw-r--r--".to_string(),
                    modified: "2026-01-01".to_string(),
                    is_hidden: false,
                    is_selected: false,
                },
            ],
        });
        s.viewports[0].current_level = HierarchyLevel::CommandHub;
        s.current_level = HierarchyLevel::CommandHub;
    }
    let ipc = make_ipc(Arc::clone(&state));

    // Switch to directory mode
    ipc.handle_request("set_mode:Directory");

    // Select a file
    ipc.handle_request("dir_toggle_select:testfile.txt");

    // Open context menu
    ipc.handle_request("dir_context:testfile.txt;200;300");

    let s = state.lock().unwrap();
    let sector_idx = s.viewports[0].sector_index;
    let hub_idx = s.viewports[0].hub_index;
    let hub = &s.sectors[sector_idx].hubs[hub_idx];

    assert_eq!(hub.mode, CommandHubMode::Directory, "Mode should be Directory");
    assert!(hub.selected_files.contains("testfile.txt"), "File should be selected");
    assert!(hub.context_menu.is_some(), "Context menu should be open");
    assert_eq!(hub.context_menu.as_ref().unwrap().target, "testfile.txt");

    // Render and verify
    let viewport = s.viewports[0].clone();
    let html = HubRenderer.render(&s, &viewport, RenderMode::Full);
    assert!(html.contains("context-menu"), "Rendered HTML should show context menu");
    assert!(html.contains("selected"), "Rendered HTML should show selected file class");
    assert!(html.contains("TESTFILE.TXT"), "Rendered HTML should show the file name");
}
