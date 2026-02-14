use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;
use tos_core::TosState;
use std::sync::{Arc, Mutex};

use tos_core::system::pty::{PtyHandle, PtyEvent};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    // 1. Initialize System State
    let state = Arc::new(Mutex::new(TosState::new()));
    let ptys: Arc<Mutex<HashMap<uuid::Uuid, PtyHandle>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Create PTYs for initial hubs
    {
        let state = state.lock().unwrap();
        for sector in &state.sectors {
            for hub in &sector.hubs {
                if let Some(pty) = PtyHandle::spawn("/usr/bin/fish", ".") {
                    ptys.lock().unwrap().insert(hub.id, pty);
                }
            }
        }
    }

    // 2. PTY Event Poller
    let state_pty = Arc::clone(&state);
    let ptys_pty = Arc::clone(&ptys);
    std::thread::spawn(move || {
        loop {
            let mut ptys = ptys_pty.lock().unwrap();
            for (hub_id, pty) in ptys.iter_mut() {
                while let Ok(event) = pty.event_rx.try_recv() {
                    let mut state = state_pty.lock().unwrap();
                    // Find the hub across all sectors
                    for sector in &mut state.sectors {
                        if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == *hub_id) {
                            match event.clone() {
                                PtyEvent::Output(data) => {
                                    hub.terminal_output.push(data);
                                    if hub.terminal_output.len() > 100 { hub.terminal_output.remove(0); }
                                }
                                PtyEvent::DirectoryChanged(path) => {
                                    println!("Hub {} directory changed to: {}", hub.id, path);
                                    // In a full implementation, we'd update the Directory view here
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            drop(ptys);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    // 3. Setup UI Thread (Tao + Wry)
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("TOS Tactical Operating System — Dream Complete")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)?;

    let webview = WebViewBuilder::new(&window)
        .with_html(include_str!("../ui/index.html"))?
        .with_ipc_handler({
            let state = Arc::clone(&state);
            let ptys = Arc::clone(&ptys);
            move |request: &str| {
                let mut state = state.lock().expect("Failed to lock state");
                
                // Handle complex commands
                if request.starts_with("set_mode:") {
                    let mode_str = &request[9..];
                    match mode_str {
                        "Command" => state.toggle_mode(tos_core::CommandHubMode::Command),
                        "Directory" => state.toggle_mode(tos_core::CommandHubMode::Directory),
                        "Activity" => state.toggle_mode(tos_core::CommandHubMode::Activity),
                        _ => {}
                    }
                } else if request.starts_with("select_sector:") {
                    if let Ok(index) = request[14..].parse::<usize>() {
                        if index < state.sectors.len() {
                            let viewport = &mut state.viewports[state.active_viewport_index];
                            viewport.sector_index = index;
                            viewport.current_level = tos_core::HierarchyLevel::CommandHub;
                            state.current_level = tos_core::HierarchyLevel::CommandHub;
                        }
                    }
                } else if request.starts_with("prompt_submit:") {
                    let cmd_full = &request[14..];
                    println!("Prompt Submitted: {}", cmd_full);
                    
                    let parts: Vec<&str> = cmd_full.split_whitespace().collect();
                    if parts.is_empty() { return; }

                    match parts[0] {
                        "zoom" => {
                            if parts.get(1) == Some(&"in") { state.zoom_in(); }
                            else if parts.get(1) == Some(&"out") { state.zoom_out(); }
                        }
                        "in" => state.zoom_in(),
                        "out" => state.zoom_out(),
                        "mode" => {
                            match parts.get(1) {
                                Some(&"command") => state.toggle_mode(tos_core::CommandHubMode::Command),
                                Some(&"directory") | Some(&"dir") => state.toggle_mode(tos_core::CommandHubMode::Directory),
                                Some(&"activity") | Some(&"apps") => state.toggle_mode(tos_core::CommandHubMode::Activity),
                                _ => {}
                            }
                        }
                        "focus" => {
                            if let Some(target) = parts.get(1) {
                                let sector = &mut state.sectors[state.viewports[state.active_viewport_index].sector_index];
                                let hub = &mut sector.hubs[state.viewports[state.active_viewport_index].hub_index];
                                if let Some(pos) = hub.applications.iter().position(|a| a.title.to_uppercase() == target.to_uppercase()) {
                                    hub.active_app_index = Some(pos);
                                    state.current_level = tos_core::HierarchyLevel::ApplicationFocus;
                                    state.viewports[state.active_viewport_index].current_level = tos_core::HierarchyLevel::ApplicationFocus;
                                }
                            }
                        }
                        _ => {
                            // Dangerous command check (Section 11.4)
                            let is_dangerous = cmd_full.contains("rm -rf") || cmd_full.contains(":(){ :|:& };:");
                            
                            let viewport = &state.viewports[state.active_viewport_index];
                            let sector = &mut state.sectors[viewport.sector_index];
                            let hub = &mut sector.hubs[viewport.hub_index];

                            if is_dangerous && hub.confirmation_required.is_none() {
                                println!("!! DANGEROUS COMMAND DETECTED: {}", cmd_full);
                                hub.confirmation_required = Some(cmd_full.to_string());
                                return;
                            }

                            // Clear confirmation and execute
                            hub.confirmation_required = None;
                            
                            // System Command: Route to PTY
                            if let Some(pty) = ptys.lock().unwrap().get(&hub.id) {
                                pty.write(&format!("{}\n", cmd_full));
                            }
                        }
                    }
                } else if request.starts_with("stage_command:") {
                    let cmd = &request[14..];
                    state.stage_command(cmd.to_string());
                } else if request.starts_with("focus_app:") {
                    let app_id_str = &request[10..];
                    let viewport = &state.viewports[state.active_viewport_index];
                    let sector = &mut state.sectors[viewport.sector_index];
                    let hub = &mut sector.hubs[viewport.hub_index];
                    if let Some(pos) = hub.applications.iter().position(|a| a.id.to_string() == app_id_str) {
                        hub.active_app_index = Some(pos);
                        state.current_level = tos_core::HierarchyLevel::ApplicationFocus;
                        state.viewports[state.active_viewport_index].current_level = tos_core::HierarchyLevel::ApplicationFocus;
                    }
                } else if request == "toggle_bezel" {
                    state.toggle_bezel();
                } else if request == "split_viewport" {
                    // Create a second viewport and a new HUB for it
                    let sector_idx = state.viewports[state.active_viewport_index].sector_index;
                    let new_hub_id = uuid::Uuid::new_v4();
                    
                    // Add new hub to the sector
                    let sector = &mut state.sectors[sector_idx];
                    sector.hubs.push(tos_core::CommandHub {
                        id: new_hub_id,
                        mode: tos_core::CommandHubMode::Command,
                        prompt: String::new(),
                        applications: Vec::new(),
                        active_app_index: None,
                        terminal_output: Vec::new(),
                        confirmation_required: None,
                    });
                    
                    let hub_idx = sector.hubs.len() - 1;

                    // Spawn PTY for new hub
                    if let Some(pty) = PtyHandle::spawn("/usr/bin/fish", ".") {
                        ptys.lock().unwrap().insert(new_hub_id, pty);
                    }

                    state.viewports.push(tos_core::Viewport {
                        id: uuid::Uuid::new_v4(),
                        sector_index: sector_idx,
                        hub_index: hub_idx,
                        current_level: tos_core::HierarchyLevel::CommandHub,
                        active_app_index: None,
                        bezel_expanded: false,
                    });
                    state.current_level = tos_core::HierarchyLevel::SplitView;
                } else {
                    match request {
                        "zoom_in" => state.zoom_in(),
                        "zoom_out" => {
                            state.zoom_out();
                            state.escape_count = 0; // Reset on manual zoom out
                        }
                        "escape_press" => {
                            state.escape_count += 1;
                            if state.escape_count >= 3 {
                                state.tactical_reset();
                            } else {
                                state.zoom_out();
                            }
                        }
                        _ => println!("Unknown IPC command: {}", request),
                    }
                }
            }
        })
        .build()?;

    let webview = Arc::new(webview);

    // 3. UI Update Loop (simple poll for this demo)
    let state_update = Arc::clone(&state);
    let webview_update = Arc::clone(&webview);
    std::thread::spawn(move || {
        loop {
            let html = {
                let state = state_update.lock().unwrap();
                state.render_current_view()
            };
            
            // Inject the new HTML into the main-content div
            let js = format!(
                r#"document.getElementById('main-content').innerHTML = `{}`; 
                   document.body.className = 'level-{:?}';
                   document.getElementById('current-location').innerText = '{:?}';
                   document.querySelectorAll('.terminal-output').forEach(el => el.scrollTop = el.scrollHeight);"#,
                html,
                { let s = state_update.lock().unwrap(); s.current_level },
                { let s = state_update.lock().unwrap(); s.current_level }
            );
            
            let _ = webview_update.evaluate_script(&js);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // 4. Main Event Loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
