use tao::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;
use tos_core::TosState;
use tos_core::system::pty::PtyHandle;
use tos_core::system::input::SemanticEvent;
use std::sync::{Arc, Mutex};
#[cfg(feature = "gamepad")]
use gilrs::{Gilrs, Event as GilrsEvent, Button};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let disable_security = args.iter().any(|arg| arg == "--disable-portal-security");

    // 1. Initialize System State
    let state = {
        let mut s = TosState::new();
        s.portal_security_bypass = disable_security;
        Arc::new(Mutex::new(s))
    };
    let ptys: Arc<Mutex<HashMap<uuid::Uuid, PtyHandle>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Create PTYs for initial hubs
    {
        let state_lock = state.lock().unwrap();
        if let Some(fish) = state_lock.shell_registry.get("fish") {
            for sector in &state_lock.sectors {
                for hub in &sector.hubs {
                    if let Some(pty) = fish.spawn(".") {
                        ptys.lock().unwrap().insert(hub.id, pty);
                    }
                }
            }
        }
    }

    // 2. Start System Pollers
    PtyHandle::poll_all(Arc::clone(&state), Arc::clone(&ptys));
    #[cfg(feature = "gamepad")]
    tos_core::system::input::poll_gamepad(Arc::clone(&state));

    // 3. Setup UI Thread (Tao + Wry)
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("TOS Tactical Operating System — Dream Complete")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)?;

    let webview = WebViewBuilder::new(&window)
        .with_html(include_str!("../ui/index.html"))
        .with_ipc_handler({
            let dispatcher = tos_core::system::ipc::IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));
            move |request: wry::http::Request<String>| {
                dispatcher.handle_request(request.body());
            }
        })
        .build()?;

    // 4. Main Event Loop with UI updates
    let state_update = Arc::clone(&state);
    let mut last_update = std::time::Instant::now();
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Periodic UI update (every 100ms)
        if last_update.elapsed().as_millis() >= 100 {
            let (html, current_level) = {
                let s = state_update.lock().unwrap();
                (s.render_current_view(), s.current_level)
            };
            
            let js = format!(
                r#"window.updateView(`{}`, "{:?}");
                   document.querySelectorAll('.terminal-output').forEach(el => el.scrollTop = el.scrollHeight);"#,
                html, current_level
            );
            
            let _ = webview.evaluate_script(&js);
            last_update = std::time::Instant::now();
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    event: tao::event::KeyEvent {
                        physical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                },
                ..
            } => {
                if let tao::keyboard::KeyCode::Unidentified(_) = physical_key {
                    // Handle unidentified keys
                } else {
                    let mut state = state_update.lock().unwrap();
                    match physical_key {
                        tao::keyboard::KeyCode::PageUp => state.handle_semantic_event(SemanticEvent::ZoomIn),
                        tao::keyboard::KeyCode::PageDown => state.handle_semantic_event(SemanticEvent::ZoomOut),
                        tao::keyboard::KeyCode::Home => state.handle_semantic_event(SemanticEvent::OpenGlobalOverview),
                        tao::keyboard::KeyCode::End => state.handle_semantic_event(SemanticEvent::TacticalReset),
                        tao::keyboard::KeyCode::F1 => state.handle_semantic_event(SemanticEvent::ModeCommand),
                        tao::keyboard::KeyCode::F2 => state.handle_semantic_event(SemanticEvent::ModeDirectory),
                        tao::keyboard::KeyCode::F3 => state.handle_semantic_event(SemanticEvent::ModeActivity),
                        tao::keyboard::KeyCode::F4 => state.handle_semantic_event(SemanticEvent::ToggleBezel),
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    });
}
