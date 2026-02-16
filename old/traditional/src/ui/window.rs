use wry::{
    application::{
        event::{Event, StartCause, WindowEvent, ElementState},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        keyboard::VirtualKeyCode,
    },
    webview::WebViewBuilder,
};
use std::sync::mpsc::{Receiver, Sender};
use crate::system::input::{InputEvent, KeyCode};
use crate::UiCommand;

// Map winit keys to our abstract KeyCode
fn map_key(k: VirtualKeyCode) -> KeyCode {
    match k {
        VirtualKeyCode::Escape => KeyCode::Escape,
        VirtualKeyCode::Space => KeyCode::Space,
        VirtualKeyCode::Return => KeyCode::Enter,
        VirtualKeyCode::Up => KeyCode::Up,
        VirtualKeyCode::Down => KeyCode::Down,
        VirtualKeyCode::Left => KeyCode::Left,
        VirtualKeyCode::Right => KeyCode::Right,
        VirtualKeyCode::Z => KeyCode::Char('z'),
        VirtualKeyCode::S => KeyCode::Char('s'),
        VirtualKeyCode::Q => KeyCode::Char('q'),
        _ => KeyCode::Unknown,
    }
}

pub fn run_ui(rx: Receiver<UiCommand>, tx: Sender<InputEvent>) -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("TOS - Traditional Navigator")
        .with_inner_size(wry::application::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?;

    let html_path = std::env::current_dir().unwrap().join("ui").join("index.html");
    let content = std::fs::read_to_string(&html_path).unwrap_or_else(|_| "<h1>Error</h1>".to_string());

    // Clone tx for IPC handler
    let ipc_tx = tx.clone();

    let webview = WebViewBuilder::new(window)?
        .with_html(content)?
        .with_ipc_handler(move |_, msg| {
            println!("[UI Bridge] IPC Message: {}", msg);
            // Example: terminal:zoom 2
            if msg.starts_with("terminal:") {
                let cmd = &msg[9..];
                let _ = ipc_tx.send(InputEvent::Command(cmd.to_string()));
            } else if msg.starts_with("zoom:") {
                let lvl = &msg[5..];
                if let Ok(l) = lvl.parse::<u8>() {
                    // Send specific zoom signal
                }
            }
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(StartCause::Init) => println!("TOS UI Initialized"),
            
            Event::MainEventsCleared => {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        UiCommand::UpdateViewport { html_content, zoom_level, is_red_alert } => {
                            // Preserve focus and state if needed by using a smart script
                            // We replace the innerHTML of the spatial-view
                            let safe_html = html_content.replace("`", "\\`").replace("$", "\\$");
                            let script = format!(
                                r#"
                                (function() {{
                                    const view = document.getElementById('spatial-view');
                                    const input = document.getElementById('terminal-input');
                                    let activeId = document.activeElement ? document.activeElement.id : null;
                                    let val = input ? input.value : "";
                                    
                                    if(view) view.innerHTML = `{0}`;
                                    
                                    // Restore state
                                    const newInput = document.getElementById('terminal-input');
                                    if(newInput && activeId === 'terminal-input') {{
                                        newInput.value = val;
                                        newInput.focus();
                                    }}
                                    
                                    document.body.className = 'zoom-level-{1}' + ({2} ? ' red-alert' : '');
                                }})();
                                "#, 
                                safe_html, zoom_level, is_red_alert
                            );
                            let _ = webview.evaluate_script(&script);
                        }
                        UiCommand::UpdateDashboard(html) => {
                             let script = format!("document.getElementById('spatial-view').innerHTML = `{}`;", html.replace("`", "\\`"));
                             let _ = webview.evaluate_script(&script);
                        }
                        UiCommand::ZoomLevel(level) => {
                            let script = format!("document.body.className = 'zoom-level-{}';", level);
                            let _ = webview.evaluate_script(&script);
                        }
                    }
                }
            },

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(vk) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            let code = map_key(vk);
                            let _ = tx.send(InputEvent::KeyDown(code));
                        }
                    }
                },
                _ => (),
            },
            _ => (),
        }
    });
}
