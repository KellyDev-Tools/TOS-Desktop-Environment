use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;
use tos_core::TosState;
use std::sync::{Arc, Mutex};

fn main() -> anyhow::Result<()> {
    // 1. Initialize System State
    let state = Arc::new(Mutex::new(TosState::new()));
    
    // 2. Setup UI Thread (Tao + Wry)
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("TOS Tactical Operating System — Dream Complete")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)?;

    let webview = WebViewBuilder::new(&window)
        .with_html(include_str!("../ui/index.html"))?
        .with_ipc_handler({
            let state = Arc::clone(&state);
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
                            state.active_sector_index = index;
                            state.current_level = tos_core::HierarchyLevel::CommandHub;
                        }
                    }
                } else if request.starts_with("prompt_submit:") {
                    let cmd = &request[14..];
                    println!("Prompt Submitted: {}", cmd);
                    // Minimal Command Handling for MVP
                    match cmd {
                        "zoom in" | "in" => state.zoom_in(),
                        "zoom out" | "out" => state.zoom_out(),
                        "mode command" => state.toggle_mode(tos_core::CommandHubMode::Command),
                        "mode directory" | "dir" => state.toggle_mode(tos_core::CommandHubMode::Directory),
                        "mode activity" | "apps" => state.toggle_mode(tos_core::CommandHubMode::Activity),
                        _ => state.set_prompt(format!("Unknown command: {}", cmd)),
                    }
                } else if request.starts_with("focus_app:") {
                    let app_id_str = &request[10..];
                    let sector = &mut state.sectors[state.active_sector_index];
                    let hub = &mut sector.hubs[sector.active_hub_index];
                    if let Some(pos) = hub.applications.iter().position(|a| a.id.to_string() == app_id_str) {
                        hub.active_app_index = Some(pos);
                        state.current_level = tos_core::HierarchyLevel::ApplicationFocus;
                    }
                } else if request == "split_viewport" {
                    // Create a second viewport for the current sector/hub
                    let sector_idx = state.viewports[state.active_viewport_index].sector_index;
                    let hub_idx = state.viewports[state.active_viewport_index].hub_index;
                    state.viewports.push(tos_core::Viewport {
                        id: uuid::Uuid::new_v4(),
                        sector_index: sector_idx,
                        hub_index: hub_idx,
                        current_level: tos_core::HierarchyLevel::CommandHub,
                        active_app_index: None,
                    });
                    state.current_level = tos_core::HierarchyLevel::SplitView;
                } else {
                    match request {
                        "zoom_in" => state.zoom_in(),
                        "zoom_out" => state.zoom_out(),
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
                   document.getElementById('current-location').innerText = '{:?}';"#,
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
