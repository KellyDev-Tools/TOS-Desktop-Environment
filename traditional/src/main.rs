use tos_comp::DesktopEnvironment;
use tos_comp::UiCommand;
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget};
use tos_comp::system::input::{InputEvent, KeyCode};
use tos_comp::system::commands::CommandParser;
use std::thread;
use std::sync::mpsc::channel;
use std::time::Duration;

#[cfg(feature = "gui")]
use tos_comp::ui::window::run_ui;

fn main() {
    println!("==========================================");
    println!("TOS Traditional Desktop Environment (Rust)");
    println!("==========================================");

    // 1. Create Communication Channels
    let (ui_tx, ui_rx) = channel::<UiCommand>();
    let (input_tx, input_rx) = channel::<InputEvent>();

    let shell_tx = ui_tx.clone();

    // 2. Spawn the "Brain" (Logic Thread)
    thread::spawn(move || {
        println!("[Brain] Logic Thread Started");
        let mut env = DesktopEnvironment::new(Some(shell_tx.clone()));

        // Initialize Dashboard
        env.dashboard.add_widget(Box::new(ClockWidget));
        env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 12, ram_usage: 45 }));
        env.dashboard.add_widget(Box::new(tos_comp::ui::dashboard::ProcessManagerWidget { processes: vec![] }));
        
        // Initial viewport send
        let _ = shell_tx.send(UiCommand::UpdateViewport {
            html_content: env.generate_viewport_html(),
            zoom_level: 1,
        });

        // Logic Loop
        let mut last_html = String::new();
        
        loop {
            env.tick(); 

            // Handle Inputs
            while let Ok(event) = input_rx.try_recv() {
                match event {
                    InputEvent::Command(cmd_str) => {
                        println!("[Brain] Received Terminal Command: {}", cmd_str);
                        let feedback = CommandParser::process(&mut env, &cmd_str);
                        println!("[Brain] Command Feedback: {}", feedback);
                        // Optional: show feedback in OSD or notification
                    },
                    InputEvent::KeyDown(key) => {
                        println!("[Brain] Key: {:?}", key);
                        match key {
                            KeyCode::Escape => {
                                env.start_zoom_morph(false);
                                env.navigator.zoom_out();
                            },
                            KeyCode::Enter => {
                                env.start_zoom_morph(true);
                                env.navigator.zoom_in(0);
                            },
                            KeyCode::Char('q') => {
                                println!("[Brain] Exiting.");
                                return;
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
            
            // Generate and send Viewport Update
            let current_html = env.generate_viewport_html();
            if current_html != last_html {
                let zoom_lvl = match env.navigator.current_level {
                    tos_comp::navigation::zoom::ZoomLevel::Level1Root => 1,
                    tos_comp::navigation::zoom::ZoomLevel::Level2Sector => 2,
                    _ => 3,
                };
                let _ = shell_tx.send(UiCommand::UpdateViewport {
                    html_content: current_html.clone(),
                    zoom_level: zoom_lvl,
                });
                last_html = current_html;
            }

            thread::sleep(Duration::from_millis(50)); // Responsive loop
        }
    });

    // 3. Run UI on Main Thread
    #[cfg(feature = "gui")]
    {
        if let Err(e) = run_ui(ui_rx, input_tx) {
            eprintln!("Failed to launch UI: {}", e);
        }
    }

    #[cfg(not(feature = "gui"))]
    {
        println!("[Headless] No GUI. Running logic only.");
        thread::sleep(Duration::from_secs(5));
        println!("[Headless] Done.");
    }
}
