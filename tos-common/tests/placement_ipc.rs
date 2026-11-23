//! Placement at the integration tier: IPC command → state
//! mutation → state JSON, driven headlessly through a real
//! Brain's IPC handler (Architecture §26.4).
//!
//! The Brain's own app catalogue comes from the XDG data dirs,
//! so the fixture plants two desktop entries in a temp
//! XDG_DATA_HOME before the first Brain is constructed (the
//! handler scans once, at startup). The "host" catalogue is
//! synced over the protocol, as a face does. Fixture app ids
//! are unique to this suite so a developer machine's real
//! catalogue can never collide with them.

use std::sync::Once;
use tos_common::brain::Brain;

static FIXTURE: Once = Once::new();

fn fixture_apps_dir() -> std::path::PathBuf {
    let base = std::env::temp_dir().join("tos-placement-ipc-fixture");
    let apps = base.join("data/applications");
    std::fs::create_dir_all(&apps).unwrap();
    std::fs::write(
        apps.join("tos-itest-editor.desktop"),
        "[Desktop Entry]\nType=Application\nName=ItestEditor\nExec=itest-editor %u\n",
    )
    .unwrap();
    std::fs::write(
        apps.join("tos-itest-viewer.desktop"),
        "[Desktop Entry]\nType=Application\nName=ItestViewer\nExec=itest-viewer\n",
    )
    .unwrap();
    let sys = base.join("sysdata/applications");
    std::fs::create_dir_all(&sys).unwrap();
    base
}

fn brain_with_host_apps() -> Brain {
    FIXTURE.call_once(|| {
        let base = fixture_apps_dir();
        std::env::set_var("XDG_DATA_HOME", base.join("data"));
        std::env::set_var("XDG_DATA_DIRS", base.join("sysdata"));
    });
    let brain = Brain::new().expect("Brain initializes");
    let res = brain.ipc.handle_request(
        "placement_sync_apps:[\
            {\"id\":\"tos-itest-editor\",\"name\":\"ItestEditor\",\"exec\":\"itest-editor %u\"},\
            {\"id\":\"tos-itest-host-app\",\"name\":\"ItestHostApp\",\"exec\":\"itest-host-app\"}\
        ]",
    );
    assert_eq!(res, "OK");
    brain
}

fn launches(brain: &Brain) -> Vec<(String, String, String)> {
    let state = brain.state.lock().unwrap();
    state
        .placement_launches
        .iter()
        .map(|l| (l.app.clone(), l.placement.clone(), l.exec.clone()))
        .collect()
}

#[tokio::test]
async fn resolution_matrix_over_ipc() {
    let brain = brain_with_host_apps();
    // Server-only → server.
    assert_eq!(
        brain.ipc.handle_request("prompt_submit:tos-itest-viewer"),
        "LAUNCHED:server"
    );
    // Host-only → host.
    assert_eq!(
        brain.ipc.handle_request("prompt_submit:tos-itest-host-app"),
        "LAUNCHED:host"
    );
    // On both → host by default.
    assert_eq!(
        brain.ipc.handle_request("prompt_submit:tos-itest-editor"),
        "LAUNCHED:host"
    );
    let got = launches(&brain);
    assert_eq!(
        got.iter().map(|l| l.1.as_str()).collect::<Vec<_>>(),
        ["server", "host", "host"]
    );
    // The directive mirrors the last launch.
    let state = brain.state.lock().unwrap();
    let directive = state.placement_directive.as_ref().unwrap();
    assert_eq!(directive.placement, "host");
    assert_eq!(directive.app, "ItestEditor");
}

#[tokio::test]
async fn non_app_submission_registers_nothing() {
    let brain = brain_with_host_apps();
    assert_eq!(
        brain.ipc.handle_request("prompt_submit:echo hello"),
        "SUBMITTED"
    );
    assert!(launches(&brain).is_empty());
    let state = brain.state.lock().unwrap();
    assert!(state.placement_directive.is_none());
}

#[tokio::test]
async fn prompt_launch_declines_non_apps_with_session() {
    let brain = brain_with_host_apps();
    assert_eq!(
        brain.ipc.handle_request("prompt_launch:ls -la"),
        "SESSION"
    );
    assert_eq!(
        brain.ipc.handle_request("prompt_launch:tos-itest-viewer"),
        "LAUNCHED:server"
    );
    assert_eq!(launches(&brain).len(), 1);
}

#[tokio::test]
async fn overrides_force_and_fail_loudly() {
    let brain = brain_with_host_apps();
    assert_eq!(
        brain.ipc.handle_request("prompt_submit:server: tos-itest-editor"),
        "LAUNCHED:server"
    );
    let res = brain
        .ipc
        .handle_request("prompt_submit:here: tos-itest-viewer");
    assert!(res.contains("not available on the host"), "got: {res}");
    let res = brain.ipc.handle_request("prompt_submit:here: nosuchapp");
    assert!(res.contains("not a known app"), "got: {res}");
    // Only the forced launch was registered.
    assert_eq!(launches(&brain).len(), 1);
}

#[tokio::test]
async fn explicit_prominence_parks_then_choose_launches() {
    let brain = brain_with_host_apps();
    {
        let mut state = brain.state.lock().unwrap();
        state
            .settings
            .global
            .insert("tos.placement.prominence".into(), "explicit".into());
    }
    let res = brain.ipc.handle_request("prompt_submit:tos-itest-editor");
    assert!(res.starts_with("PLACEMENT_PENDING:"), "got: {res}");
    let id = res.trim_start_matches("PLACEMENT_PENDING:").to_string();
    {
        let state = brain.state.lock().unwrap();
        let pending = state.placement_pending.as_ref().unwrap();
        assert_eq!(pending.app_name, "ItestEditor");
        assert!(pending.host_exec.is_some());
        assert!(pending.server_exec.is_some());
    }
    // A wrong id is refused and the pending launch survives.
    let res = brain.ipc.handle_request("placement_choose:bogus;server");
    assert!(res.starts_with("ERROR"), "got: {res}");
    assert!(brain.state.lock().unwrap().placement_pending.is_some());
    // The right id completes the launch on the chosen side.
    assert_eq!(
        brain.ipc.handle_request(&format!("placement_choose:{id};server")),
        "LAUNCHED:server"
    );
    let state = brain.state.lock().unwrap();
    assert!(state.placement_pending.is_none());
    assert_eq!(state.placement_launches.len(), 1);
    assert_eq!(state.placement_launches[0].placement, "server");
}

#[tokio::test]
async fn choose_rejects_session_and_unknown_sides() {
    let brain = brain_with_host_apps();
    let res = brain.ipc.handle_request("placement_choose:x;session");
    assert!(res.starts_with("ERROR"), "got: {res}");
    let res = brain.ipc.handle_request("placement_choose:x;moon");
    assert!(res.starts_with("ERROR"), "got: {res}");
    let res = brain.ipc.handle_request("placement_choose:x;host");
    assert!(res.contains("No placement is waiting"), "got: {res}");
}

#[tokio::test]
async fn relaunch_flips_sides_and_refuses_missing_apps() {
    let brain = brain_with_host_apps();
    assert_eq!(
        brain.ipc.handle_request("prompt_submit:tos-itest-host-app"),
        "LAUNCHED:host"
    );
    assert_eq!(
        brain.ipc.handle_request("prompt_submit:tos-itest-editor"),
        "LAUNCHED:host"
    );
    let (host_only_id, editor_id) = {
        let state = brain.state.lock().unwrap();
        (
            state.placement_launches[0].id.clone(),
            state.placement_launches[1].id.clone(),
        )
    };
    // The server lacks the host-only app: loud refusal, no launch.
    let res = brain
        .ipc
        .handle_request(&format!("placement_relaunch:{host_only_id}"));
    assert!(res.contains("not available on the server"), "got: {res}");
    assert_eq!(launches(&brain).len(), 2);
    // The editor is on both machines: flips to the server.
    assert_eq!(
        brain.ipc.handle_request(&format!("placement_relaunch:{editor_id}")),
        "LAUNCHED:server"
    );
    let res = brain.ipc.handle_request("placement_relaunch:nope");
    assert!(res.contains("Unknown launch id"), "got: {res}");
}

#[tokio::test]
async fn registry_is_capped_and_sync_replaces_the_catalogue() {
    let brain = brain_with_host_apps();
    for _ in 0..25 {
        brain.ipc.handle_request("prompt_submit:tos-itest-viewer");
    }
    assert_eq!(launches(&brain).len(), 20);
    // A fresh sync replaces the host catalogue wholesale: the
    // host-only app stops resolving as a launch at all.
    assert_eq!(
        brain.ipc.handle_request("placement_sync_apps:[]"),
        "OK"
    );
    assert_eq!(
        brain.ipc.handle_request("prompt_launch:tos-itest-host-app"),
        "SESSION"
    );
    // A malformed catalogue is refused and changes nothing.
    let res = brain.ipc.handle_request("placement_sync_apps:{nope");
    assert!(res.starts_with("ERROR"), "got: {res}");
}
