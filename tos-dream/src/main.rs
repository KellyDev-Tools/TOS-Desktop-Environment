use tao::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState},
    window::WindowBuilder,
};
use wry::WebViewBuilder;
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
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

    // Serve UI files via a custom "tos://" protocol.
    // - with_html() breaks relative CSS/JS paths (no base URL).
    // - file:// URLs crash wry's IPC handler (http::Uri doesn't support file:// scheme).
    // - A custom protocol gives valid URIs for IPC AND resolves relative paths correctly.
    let ui_base = concat!(env!("CARGO_MANIFEST_DIR"), "/ui");

    let custom_protocol_handler = move |request: wry::http::Request<Vec<u8>>| {
        let path = request.uri().path();
        let path = if path.is_empty() || path == "/" { "/index.html" } else { path };

        let file_path = format!("{}{}", ui_base, path);
        let content = std::fs::read(&file_path).unwrap_or_else(|_| {
            format!("<h1>404 Not Found</h1><p>{}</p>", file_path).into_bytes()
        });

        let mime_type = if path.ends_with(".html") {
            "text/html"
        } else if path.ends_with(".css") {
            "text/css"
        } else if path.ends_with(".js") {
            "application/javascript"
        } else if path.ends_with(".png") {
            "image/png"
        } else if path.ends_with(".svg") {
            "image/svg+xml"
        } else if path.ends_with(".woff2") {
            "font/woff2"
        } else {
            "application/octet-stream"
        };

        wry::http::Response::builder()
            .header("Content-Type", mime_type)
            .body(std::borrow::Cow::Owned(content))
            .unwrap()
    };

    // On Linux, use the GTK-specific builder to support both Wayland and X11.
    // WebViewBuilder::new(&window) fails on Wayland because the raw window handle
    // kind (GdkWayland) is not supported by wry. Using new_gtk() with the tao
    // window's built-in GTK VBox container avoids this issue entirely.
    #[cfg(target_os = "linux")]
    let webview = {
        let vbox = window.default_vbox().expect("tao window should have a default GTK VBox");
        WebViewBuilder::new_gtk(vbox)
            .with_custom_protocol("tos".into(), custom_protocol_handler)
            .with_url("tos://localhost/index.html")
            .with_ipc_handler({
                let dispatcher = tos_core::system::ipc::IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));
                move |request: wry::http::Request<String>| {
                    dispatcher.handle_request(request.body());
                }
            })
            .build()?
    };
    #[cfg(not(target_os = "linux"))]
    let webview = WebViewBuilder::new(&window)
        .with_custom_protocol("tos".into(), custom_protocol_handler)
        .with_url("tos://localhost/index.html")
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
    // Section 14: Track modifiers for Super+Backspace / Super+Alt+Backspace
    let mut modifiers = ModifiersState::empty();

    let ptys_cleanup = Arc::clone(&ptys);

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
            } => {
                // Explicitly shut down all PTY handles before exiting.
                // Then use process::exit() to avoid the destructor cascade
                // where Rust's drop order races with WebKit2GTK's internal
                // cleanup, causing "double free or corruption" on Linux.
                if let Ok(mut ptys_lock) = ptys_cleanup.lock() {
                    ptys_lock.clear(); // Drops all PtyHandles, triggering graceful shutdown
                }
                
                // Save state on exit
                state_update.lock().unwrap().save();

                // Give PTY threads time to close their file descriptors
                std::thread::sleep(std::time::Duration::from_millis(100));
                std::process::exit(0);
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(m),
                ..
            } => modifiers = m,
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
                if let KeyCode::Unidentified(_) = physical_key {
                    // Handle unidentified keys
                } else {
                    let mut state = state_update.lock().unwrap();
                    match physical_key {
                        KeyCode::PageUp => state.handle_semantic_event(SemanticEvent::ZoomIn),
                        KeyCode::PageDown => state.handle_semantic_event(SemanticEvent::ZoomOut),
                        KeyCode::Home => state.handle_semantic_event(SemanticEvent::OpenGlobalOverview),
                        KeyCode::End => state.handle_semantic_event(SemanticEvent::TacticalReset),
                        KeyCode::F1 => state.handle_semantic_event(SemanticEvent::ModeCommand),
                        KeyCode::F2 => state.handle_semantic_event(SemanticEvent::ModeDirectory),
                        KeyCode::F3 => state.handle_semantic_event(SemanticEvent::ModeActivity),
                        KeyCode::F4 => state.handle_semantic_event(SemanticEvent::ToggleBezel),
                        // Section 14: Super+Alt+Backspace = Level 2 system reset, Super+Backspace = Level 1 sector reset
                        KeyCode::Backspace => {
                            if modifiers.super_key() && modifiers.alt_key() {
                                state.handle_semantic_event(SemanticEvent::SystemReset);
                            } else if modifiers.super_key() {
                                state.handle_semantic_event(SemanticEvent::TacticalReset);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    });
}
