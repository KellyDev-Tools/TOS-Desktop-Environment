use wry::{
    application::{
        event::{Event, StartCause, WindowEvent, ElementState},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        keyboard::VirtualKeyCode,
    },
    webview::WebViewBuilder,
};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
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
        .with_title("TOS - Native Compositor")
        .with_inner_size(wry::application::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?;

    let html_path = std::env::current_dir().unwrap().join("ui").join("index.html");
    let content = std::fs::read_to_string(&html_path).unwrap_or_else(|_| "<h1>Error</h1>".to_string());

    let webview = WebViewBuilder::new(window)?
        .with_html(content)?
        .with_ipc_handler(|_, msg| {
            println!("[UI Bridge] Message from JS: {}", msg);
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(StartCause::Init) => println!("TOS UI Initialized"),
            
            Event::MainEventsCleared => {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        UiCommand::UpdateDashboard(html) => {
                            let safe_html = html.replace("`", "\\`");
                            let script = format!(
                                "const grid = document.getElementById('dashboard-grid'); if(grid) grid.innerHTML = `{}`;", 
                                safe_html
                            );
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
