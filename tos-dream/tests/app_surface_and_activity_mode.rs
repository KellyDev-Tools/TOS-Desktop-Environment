//! Tests for P0 Items 2 & 3:
//!   - Item 2: App surface is not a placeholder (§4) — AppRenderer shows real data
//!   - Item 3: Activity Mode CPU/MEM are real numbers (§3.3) — HubRenderer reads /proc
//!
//! Covers:
//! - Unit: proc.rs reads real /proc data for live processes
//! - Component: AppRenderer renders terminal content for Shell apps, real PID/MEM for others
//! - Component: HubRenderer Activity Mode shows real CPU/MEM from /proc
//! - Integration: Full render pipeline through TosState::render_viewport

use tos_core::{TosState, HierarchyLevel, RenderMode, CommandHubMode, DecorationPolicy};
use tos_core::ui::render::{ViewRenderer, app::AppRenderer, hub::HubRenderer};
use tos_core::system::proc::{get_process_stats, get_process_buffer_sample};

// ─── Unit Tests: proc.rs reads real /proc data ────────────────────────────────

#[test]
fn test_get_process_stats_for_self() {
    let self_pid = std::process::id();
    let stats = get_process_stats(self_pid).expect("Should read /proc stats for self");

    assert_eq!(stats.pid, self_pid);
    assert!(stats.memory_bytes > 0, "Memory should be > 0 for a running process");
    assert!(stats.uptime_seconds >= 0.0, "Uptime should be non-negative");
    assert!(stats.cpu_usage >= 0.0, "CPU usage should be non-negative");
}

#[test]
fn test_get_process_stats_cmdline_not_empty() {
    let self_pid = std::process::id();
    let stats = get_process_stats(self_pid).expect("Should read stats for self");
    assert!(!stats.cmdline.is_empty(), "cmdline should not be empty for a running process");
}

#[test]
fn test_get_process_stats_invalid_pid_returns_error() {
    let result = get_process_stats(0);
    assert!(result.is_err(), "PID 0 should return an error");
}

#[test]
fn test_get_process_stats_nonexistent_pid_returns_error() {
    let result = get_process_stats(u32::MAX);
    assert!(result.is_err(), "Non-existent PID should return an error");
}

#[test]
fn test_get_process_buffer_sample_for_self() {
    let self_pid = std::process::id();
    let buffer = get_process_buffer_sample(self_pid);
    assert!(buffer.len() >= 512, "Buffer should be at least 512 bytes");
    assert!(buffer.iter().any(|&b| b != 0), "Buffer should contain non-zero data");
}

#[test]
fn test_get_process_buffer_sample_invalid_pid_returns_padded_buffer() {
    let buffer = get_process_buffer_sample(u32::MAX);
    assert!(buffer.len() >= 512, "Buffer should still be 512 bytes even for invalid PID");
}

#[test]
fn test_process_stats_memory_in_bytes_not_kb() {
    let self_pid = std::process::id();
    let stats = get_process_stats(self_pid).unwrap();
    // A running Rust test binary should use at least 1MB of RAM
    assert!(
        stats.memory_bytes >= 1024 * 1024,
        "memory_bytes should be at least 1MB for a Rust test binary, got {} bytes",
        stats.memory_bytes
    );
}

#[test]
fn test_proc_stats_for_spawned_process() {
    let child = std::process::Command::new("sleep")
        .arg("10")
        .spawn()
        .expect("Failed to spawn sleep");
    let pid = child.id();

    std::thread::sleep(std::time::Duration::from_millis(50));

    let result = get_process_stats(pid);
    let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).status();

    let stats = result.expect("Should read stats for spawned process");
    assert_eq!(stats.pid, pid);
    assert!(stats.memory_bytes > 0, "Spawned process should have non-zero memory");
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_app(title: &str, app_class: &str, pid: Option<u32>, is_dummy: bool) -> tos_core::Application {
    tos_core::Application {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        app_class: app_class.to_string(),
        is_minimized: false,
        pid,
        icon: Some("🔬".to_string()),
        is_dummy,
        settings: std::collections::HashMap::new(),
        thumbnail: None,
        decoration_policy: DecorationPolicy::Native,
        bezel_actions: vec![],
    }
}

fn make_state_with_app(app_class: &str, pid: Option<u32>) -> (TosState, tos_core::Viewport) {
    let mut state = TosState::new();
    state.current_level = HierarchyLevel::ApplicationFocus;

    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    state.sectors[sector_idx].hubs[hub_idx].applications.clear();
    state.sectors[sector_idx].hubs[hub_idx].applications.push(
        make_app("Test App", app_class, pid, false)
    );
    state.sectors[sector_idx].hubs[hub_idx].active_app_index = Some(0);
    state.viewports[0].active_app_index = Some(0);
    state.viewports[0].current_level = HierarchyLevel::ApplicationFocus;

    let viewport = state.viewports[0].clone();
    (state, viewport)
}

fn make_state_activity_mode(pid: Option<u32>, is_dummy: bool) -> (TosState, tos_core::Viewport) {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Activity;
    state.sectors[sector_idx].hubs[hub_idx].applications.clear();
    state.sectors[sector_idx].hubs[hub_idx].applications.push(
        make_app("Test Process", "TestClass", pid, is_dummy)
    );
    state.viewports[0].current_level = HierarchyLevel::CommandHub;

    let viewport = state.viewports[0].clone();
    (state, viewport)
}

// ─── Component Tests: AppRenderer ─────────────────────────────────────────────

#[test]
fn test_app_renderer_shell_app_shows_terminal_content() {
    let (mut state, viewport) = make_state_with_app("Shell", None);
    let sector_idx = viewport.sector_index;
    let hub_idx = viewport.hub_index;

    state.sectors[sector_idx].hubs[hub_idx].terminal_output = vec![
        "$ ls -la".to_string(),
        "total 42".to_string(),
    ];

    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("terminal-content"), "Shell app should render terminal-content class");
    assert!(html.contains("$ ls -la"), "Terminal output should appear in render");
    assert!(html.contains("total 42"), "All terminal lines should appear");
    assert!(!html.contains("INITIALIZING SUBSYSTEMS"), "Shell app should not show placeholder boot text");
}

#[test]
fn test_app_renderer_terminal_class_shows_terminal_content() {
    let (state, viewport) = make_state_with_app("terminal", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("terminal-content"), "App with 'terminal' class should render terminal view");
}

#[test]
fn test_app_renderer_non_shell_app_shows_real_pid() {
    let self_pid = std::process::id();
    let (state, viewport) = make_state_with_app("Spectrometer", Some(self_pid));
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(
        html.contains(&self_pid.to_string()),
        "Non-shell app should show real PID {}", self_pid
    );
    assert!(!html.contains("TOS-SYS"), "App with real PID should not show TOS-SYS placeholder");
}

#[test]
fn test_app_renderer_non_shell_app_no_pid_shows_tos_sys() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("TOS-SYS"), "App without PID should show TOS-SYS placeholder");
}

#[test]
fn test_app_renderer_non_shell_app_with_pid_shows_real_memory() {
    let self_pid = std::process::id();
    let (state, viewport) = make_state_with_app("Spectrometer", Some(self_pid));
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);

    // Should show real memory (not ---MB)
    assert!(
        !html.contains("---MB"),
        "App with real PID should show real memory, not ---MB"
    );
    assert!(html.contains("LOAD:"), "Footer should contain LOAD: stat");
    assert!(html.contains("MB"), "Memory should be displayed in MB");
}

#[test]
fn test_app_renderer_non_shell_app_no_pid_shows_placeholder_memory() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("---MB"), "App without PID should show ---MB placeholder for memory");
}

#[test]
fn test_app_renderer_shows_app_title_and_class_in_bezel() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("TEST APP"), "App title should appear uppercased in bezel");
    assert!(html.contains("SPECTROMETER"), "App class should appear uppercased in bezel");
}

#[test]
fn test_app_renderer_shows_bezel_controls() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("ZOOM OUT"), "Bezel should have ZOOM OUT button");
    assert!(html.contains("SPLIT VIEW"), "Bezel should have SPLIT VIEW button");
    assert!(html.contains("EXPORT PORTAL"), "Bezel should have portal button");
    assert!(html.contains("PRIORITY"), "Bezel should have PRIORITY slider");
    assert!(html.contains("GAIN"), "Bezel should have GAIN slider");
    assert!(html.contains("SENSITIVITY"), "Bezel should have SENSITIVITY slider");
}

#[test]
fn test_app_renderer_sliders_use_app_settings() {
    let (mut state, viewport) = make_state_with_app("Spectrometer", None);
    let sector_idx = viewport.sector_index;
    let hub_idx = viewport.hub_index;

    state.sectors[sector_idx].hubs[hub_idx].applications[0]
        .settings.insert("priority".to_string(), 8.0);
    state.sectors[sector_idx].hubs[hub_idx].applications[0]
        .settings.insert("gain".to_string(), 42.0);

    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("value=\"8\""), "Priority slider should use app setting value 8");
    assert!(html.contains("value=\"42\""), "Gain slider should use app setting value 42");
}

#[test]
fn test_app_renderer_shows_version_in_surface() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);
    let version = env!("CARGO_PKG_VERSION");
    assert!(html.contains(version), "App surface should show package version {}", version);
}

#[test]
fn test_app_renderer_shows_uuid_short_in_surface() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let sector_idx = viewport.sector_index;
    let hub_idx = viewport.hub_index;
    let app_id = state.sectors[sector_idx].hubs[hub_idx].applications[0].id;
    let uuid_short = &app_id.to_string()[..8];

    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains(uuid_short), "App surface should show short UUID");
}

#[test]
fn test_app_renderer_render_mode_appears_in_class() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("render-Full"), "Render mode should appear as CSS class");
}

#[test]
fn test_app_renderer_throttled_mode_in_class() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Throttled);
    assert!(html.contains("render-Throttled"), "Throttled render mode should appear as CSS class");
}

#[test]
fn test_app_renderer_static_mode_in_class() {
    let (state, viewport) = make_state_with_app("Spectrometer", None);
    let html = AppRenderer.render(&state, &viewport, RenderMode::Static);
    assert!(html.contains("render-Static"), "Static render mode should appear as CSS class");
}

// ─── Component Tests: HubRenderer Activity Mode ───────────────────────────────

#[test]
fn test_hub_activity_mode_shows_real_cpu_for_live_pid() {
    let self_pid = std::process::id();
    let (state, viewport) = make_state_activity_mode(Some(self_pid), false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("CPU:"), "Activity mode should show CPU stat");
    assert!(
        !html.contains("CPU: ---"),
        "Activity mode with real PID should not show CPU: --- placeholder"
    );
}

#[test]
fn test_hub_activity_mode_shows_real_mem_for_live_pid() {
    let self_pid = std::process::id();
    let (state, viewport) = make_state_activity_mode(Some(self_pid), false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("MEM:"), "Activity mode should show MEM stat");
    assert!(
        !html.contains("MEM: ---"),
        "Activity mode with real PID should not show MEM: --- placeholder"
    );
    assert!(html.contains("MB"), "MEM stat should be in MB");
}

#[test]
fn test_hub_activity_mode_shows_real_pid_label() {
    let self_pid = std::process::id();
    let (state, viewport) = make_state_activity_mode(Some(self_pid), false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(
        html.contains(&format!("PID: {}", self_pid)),
        "Activity mode should show real PID {}", self_pid
    );
}

#[test]
fn test_hub_activity_mode_no_pid_shows_placeholder_stats() {
    let (state, viewport) = make_state_activity_mode(None, false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("CPU: ---"), "App without PID should show CPU: --- placeholder");
    assert!(html.contains("MEM: ---"), "App without PID should show MEM: --- placeholder");
    assert!(html.contains("PID: ---"), "App without PID should show PID: --- placeholder");
}

#[test]
fn test_hub_activity_mode_dummy_app_shows_tos_stats() {
    let (state, viewport) = make_state_activity_mode(None, true);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("CPU: 0.0%"), "Dummy app should show CPU: 0.0%");
    assert!(html.contains("MEM: <1MB"), "Dummy app should show MEM: <1MB");
    assert!(html.contains("PID: [TOS]"), "Dummy app should show PID: [TOS]");
}

#[test]
fn test_hub_activity_mode_shows_kill_and_sigint_buttons() {
    let (state, viewport) = make_state_activity_mode(Some(std::process::id()), false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("KILL"), "Activity mode should have KILL button");
    assert!(html.contains("SIGINT"), "Activity mode should have SIGINT button");
}

#[test]
fn test_hub_activity_mode_shows_new_process_button() {
    let (state, viewport) = make_state_activity_mode(None, false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("NEW PROCESS"), "Activity mode should have + NEW PROCESS button");
}

#[test]
fn test_hub_activity_mode_shows_app_icon() {
    let (state, viewport) = make_state_activity_mode(None, false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("🔬"), "Activity mode should show the app icon");
}

#[test]
fn test_hub_activity_mode_shows_section_titles() {
    let (state, viewport) = make_state_activity_mode(None, false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("ACTIVE PROCESSES"), "Should show ACTIVE PROCESSES section");
    assert!(html.contains("MODULE DATA FEEDS"), "Should show MODULE DATA FEEDS section");
    assert!(html.contains("SECTOR TEMPLATES"), "Should show SECTOR TEMPLATES section");
}

#[test]
fn test_hub_activity_mode_class_applied() {
    let (state, viewport) = make_state_activity_mode(None, false);
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);
    assert!(html.contains("mode-activity"), "Activity mode should apply mode-activity CSS class");
}

#[test]
fn test_hub_activity_mode_multi_select_shows_batch_toolbar() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Activity;
    let app = make_app("App", "Test", None, false);
    let app_id = app.id;
    state.sectors[sector_idx].hubs[hub_idx].applications.push(app);
    state.sectors[sector_idx].hubs[hub_idx].selected_files.insert(app_id.to_string());
    state.viewports[0].current_level = HierarchyLevel::CommandHub;

    let viewport = state.viewports[0].clone();
    let html = HubRenderer.render(&state, &viewport, RenderMode::Full);

    assert!(html.contains("APPS SELECTED"), "Should show batch toolbar when apps are selected");
    assert!(html.contains("app_batch_kill"), "Batch toolbar should have kill action");
    assert!(html.contains("app_batch_signal:INT"), "Batch toolbar should have SIGINT action");
}

// ─── Integration Tests: through TosState::render_viewport ────────────────────

#[test]
fn test_render_viewport_app_focus_includes_real_pid() {
    let mut state = TosState::new();
    let self_pid = std::process::id();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    state.sectors[sector_idx].hubs[hub_idx].applications.clear();
    state.sectors[sector_idx].hubs[hub_idx].applications.push(
        make_app("Integration Test App", "TestClass", Some(self_pid), false)
    );
    state.sectors[sector_idx].hubs[hub_idx].active_app_index = Some(0);
    state.viewports[0].active_app_index = Some(0);
    state.viewports[0].current_level = HierarchyLevel::ApplicationFocus;
    state.current_level = HierarchyLevel::ApplicationFocus;

    let viewport = state.viewports[0].clone();
    let html = state.render_viewport(&viewport);

    assert!(
        html.contains(&self_pid.to_string()),
        "Full render should include real PID {}", self_pid
    );
}

#[test]
fn test_render_viewport_activity_mode_includes_real_stats() {
    let mut state = TosState::new();
    let self_pid = std::process::id();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    state.sectors[sector_idx].hubs[hub_idx].mode = CommandHubMode::Activity;
    state.sectors[sector_idx].hubs[hub_idx].applications.clear();
    state.sectors[sector_idx].hubs[hub_idx].applications.push(
        make_app("Live Process", "TestClass", Some(self_pid), false)
    );
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    state.current_level = HierarchyLevel::CommandHub;

    let viewport = state.viewports[0].clone();
    let html = state.render_viewport(&viewport);

    assert!(html.contains("MEM:"), "Full render should include MEM stat");
    assert!(html.contains("CPU:"), "Full render should include CPU stat");
    assert!(
        html.contains(&format!("PID: {}", self_pid)),
        "Full render should include real PID"
    );
    // Verify it's NOT showing placeholder dashes
    assert!(!html.contains("MEM: ---"), "Should not show MEM: --- with real PID");
    assert!(!html.contains("CPU: ---"), "Should not show CPU: --- with real PID");
}

#[test]
fn test_render_viewport_shell_app_shows_terminal_not_placeholder() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    state.sectors[sector_idx].hubs[hub_idx].applications.clear();
    state.sectors[sector_idx].hubs[hub_idx].applications.push(
        make_app("My Shell", "Shell", None, false)
    );
    state.sectors[sector_idx].hubs[hub_idx].terminal_output = vec!["$ echo hello".to_string()];
    state.sectors[sector_idx].hubs[hub_idx].active_app_index = Some(0);
    state.viewports[0].active_app_index = Some(0);
    state.viewports[0].current_level = HierarchyLevel::ApplicationFocus;
    state.current_level = HierarchyLevel::ApplicationFocus;

    let viewport = state.viewports[0].clone();
    let html = state.render_viewport(&viewport);

    assert!(html.contains("terminal-content"), "Shell app should render terminal view");
    assert!(html.contains("$ echo hello"), "Terminal output should appear");
    assert!(!html.contains("INITIALIZING SUBSYSTEMS"), "Should not show placeholder text for shell");
}
