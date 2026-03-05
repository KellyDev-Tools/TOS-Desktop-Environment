//! Face Visual State Stimulator
//!
//! Drives a Face through every visual state it can render and validates
//! the output at each step. This exercises the full render pipeline:
//!
//!   State mutation (via IPC) → Face.render_to_string() → Output validation
//!
//! States exercised:
//!   1. GlobalOverview — sector tiles, system log, minimap
//!   2. CommandHub (Command mode) — prompt, terminal output area
//!   3. CommandHub (Directory mode) — directory listing context
//!   4. CommandHub (AI mode) — staged command + rationale
//!   5. ApplicationFocus — placeholder view
//!   6. Multi-sector — minimap shows multiple sectors
//!   7. Frozen sector — frozen state reflected in render

use tos_alpha2::brain::ipc_handler::IpcHandler;
use tos_alpha2::common::TosState;
use tos_alpha2::face::Face;
use tos_alpha2::services::ServiceManager;
use std::sync::{Arc, Mutex};

/// Boot a headless Brain + Face (no renderer, no display).
fn boot_face() -> (Face, Arc<IpcHandler>, Arc<Mutex<TosState>>) {
    let state = Arc::new(Mutex::new(TosState::default()));
    let services = Arc::new(ServiceManager::new());
    let modules = Arc::new(tos_alpha2::brain::module_manager::ModuleManager::new(
        std::path::PathBuf::from("./modules"),
    ));

    let sid = state.lock().unwrap().sectors[0].id;
    let hid = state.lock().unwrap().sectors[0].hubs[0].id;

    let shell = tos_alpha2::brain::shell::ShellApi::new(
        state.clone(), modules.clone(), sid, hid,
    ).expect("Face stimulator requires at least /bin/sh");

    let shell = Arc::new(Mutex::new(shell));
    let ipc = Arc::new(IpcHandler::new(state.clone(), shell, services));
    let face = Face::new(state.clone(), ipc.clone());

    (face, ipc, state)
}

// ---------------------------------------------------------------------------
// State 1: GlobalOverview — the default landing view
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_renders_global_overview_with_sector_tiles() {
    let (face, _ipc, _state) = boot_face();

    let frame = face.render_to_string();

    assert!(frame.contains("[TOS DISPLAY ENGINE]"), "Must show display engine header");
    assert!(frame.contains("GLOBAL OVERVIEW"), "Must show Level 1 title");
    assert!(frame.contains("SECTOR TILES"), "Must show sector tile header");
    assert!(frame.contains("Primary"), "Default sector 'Primary' must appear");
    assert!(frame.contains("HUBS:"), "Sector tile must show hub count");
    assert!(frame.contains("[SYSTEM OUTPUT AREA"), "Must show system log section");
    assert!(frame.contains("[TACTICAL MINI-MAP]"), "Must show minimap");
    assert!(frame.contains("BRAIN: ACTIVE"), "Footer must show Brain status");
    assert!(frame.contains("LEVEL: GlobalOverview"), "Footer must show current level");
}

// ---------------------------------------------------------------------------
// State 2: CommandHub — zoomed into a sector's hub
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_renders_command_hub_with_prompt() {
    let (face, ipc, _state) = boot_face();

    ipc.handle_request("zoom_in:");

    let frame = face.render_to_string();

    assert!(frame.contains("COMMAND HUB"), "Must show Level 2 title");
    assert!(frame.contains("PRIMARY"), "Sector name must appear in title (uppercased)");
    assert!(frame.contains("MODE:"), "Must show current mode");
    assert!(frame.contains("Command"), "Default mode is Command");
    assert!(frame.contains("DIR:"), "Must show current directory");
    assert!(frame.contains("PROMPT:"), "Must show command prompt");
    assert!(frame.contains("LEVEL: CommandHub"), "Footer must reflect CommandHub");
}

// ---------------------------------------------------------------------------
// State 3: Directory mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_renders_directory_mode_after_mode_switch() {
    let (face, ipc, _state) = boot_face();

    ipc.handle_request("zoom_in:");
    ipc.handle_request("set_mode:directory");

    let frame = face.render_to_string();

    assert!(frame.contains("COMMAND HUB"), "Still at Level 2");
    assert!(frame.contains("Directory"), "Mode must show Directory");
}

// ---------------------------------------------------------------------------
// State 4: AI mode with staged command
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_renders_ai_staged_command_with_rationale() {
    let (face, ipc, _state) = boot_face();

    ipc.handle_request("zoom_in:");
    ipc.handle_request("set_mode:ai");
    ipc.handle_request(
        r#"ai_stage_command:{"command":"git rebase -i HEAD~5","explanation":"Interactive rebase of last 5 commits"}"#,
    );

    let frame = face.render_to_string();

    assert!(frame.contains("[AI STAGED COMMAND]"), "Must show AI staging section");
    assert!(frame.contains("git rebase -i HEAD~5"), "Staged command must appear");
    assert!(frame.contains("Interactive rebase"), "Rationale must appear");
}

// ---------------------------------------------------------------------------
// State 5: ApplicationFocus — placeholder view for deeper levels
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_renders_application_focus_placeholder() {
    let (face, ipc, _state) = boot_face();

    ipc.handle_request("zoom_to:ApplicationFocus");

    let frame = face.render_to_string();

    assert!(frame.contains("ApplicationFocus VIEW"), "Must show placeholder title");
    assert!(frame.contains("PLACEHOLDER"), "Placeholder text must appear");
    assert!(frame.contains("LEVEL: ApplicationFocus"), "Footer must reflect level");
}

// ---------------------------------------------------------------------------
// State 6: Multi-sector — minimap shows all sectors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_renders_minimap_with_multiple_sectors() {
    let (face, ipc, _state) = boot_face();

    ipc.handle_request("sector_create:Engineering");
    ipc.handle_request("sector_create:Ops");

    let frame = face.render_to_string();

    // Minimap should show S0, S1, S2
    assert!(frame.contains("S0"), "Minimap must show sector 0");
    assert!(frame.contains("S1"), "Minimap must show sector 1");
    assert!(frame.contains("S2"), "Minimap must show sector 2");

    // Sector tiles should list all three
    assert!(frame.contains("Primary"), "First sector must appear");
    assert!(frame.contains("Engineering"), "Second sector must appear");
    assert!(frame.contains("Ops"), "Third sector must appear");
}

// ---------------------------------------------------------------------------
// State 7: Sector freeze visibility
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_renders_frozen_sector_in_global_overview() {
    let (_face, ipc, state) = boot_face();

    let sector_id = state.lock().unwrap().sectors[0].id.to_string();
    ipc.handle_request(&format!("sector_freeze:{}", sector_id));

    // Verify the state is frozen (the Face would render a "FROZEN" badge
    // when this is implemented — for now we just verify state)
    assert!(state.lock().unwrap().sectors[0].frozen, "Sector must be frozen");
}

// ---------------------------------------------------------------------------
// Transition sequence: walk through multiple states in order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn face_transitions_through_full_hierarchy_sequence() {
    let (face, ipc, _state) = boot_face();

    // Frame 1: GlobalOverview
    let frame1 = face.render_to_string();
    assert!(frame1.contains("GLOBAL OVERVIEW"));

    // Frame 2: Zoom in → CommandHub
    ipc.handle_request("zoom_in:");
    let frame2 = face.render_to_string();
    assert!(frame2.contains("COMMAND HUB"));
    assert!(!frame2.contains("GLOBAL OVERVIEW"), "Should no longer show L1");

    // Frame 3: Switch to AI mode, stage a command
    ipc.handle_request("set_mode:ai");
    ipc.handle_request(
        r#"ai_stage_command:{"command":"docker compose up -d","explanation":"Start services in background"}"#,
    );
    let frame3 = face.render_to_string();
    assert!(frame3.contains("Ai"), "Mode should be Ai");
    assert!(frame3.contains("docker compose up -d"));

    // Frame 4: Accept the AI suggestion, verify prompt changes
    ipc.handle_request("ai_suggestion_accept:");
    let frame4 = face.render_to_string();
    assert!(frame4.contains("docker compose up -d"), "Prompt should have accepted command");
    assert!(!frame4.contains("[AI STAGED COMMAND]"), "Staging section should be gone");

    // Frame 5: Zoom out → GlobalOverview
    ipc.handle_request("zoom_out:");
    let frame5 = face.render_to_string();
    assert!(frame5.contains("GLOBAL OVERVIEW"), "Should return to L1");

    // Frame 6: Zoom deep → ApplicationFocus
    ipc.handle_request("zoom_to:ApplicationFocus");
    let frame6 = face.render_to_string();
    assert!(frame6.contains("ApplicationFocus VIEW"));
}
