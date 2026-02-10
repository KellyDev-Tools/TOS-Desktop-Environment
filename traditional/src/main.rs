use tos_comp::DesktopEnvironment;
use tos_comp::UiCommand;
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget};
use tos_comp::system::input::{InputEvent, KeyCode};
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
        
        let _ = shell_tx.send(UiCommand::UpdateDashboard(env.dashboard.render_all_html()));

        // Logic Loop
        loop {
            while let Ok(event) = input_rx.try_recv() {
                match event {
                    InputEvent::KeyDown(key) => {
                        println!("[Brain] Key: {:?}", key);
                        match key {
                            KeyCode::Escape => {
                                env.navigator.zoom_out();
                                let level = match env.navigator.current_level {
                                    tos_comp::navigation::zoom::ZoomLevel::Level1Root => 1,
                                    tos_comp::navigation::zoom::ZoomLevel::Level2Sector => 2,
                                    tos_comp::navigation::zoom::ZoomLevel::Level3Focus => 3,
                                    tos_comp::navigation::zoom::ZoomLevel::Level3aPicker => 3,
                                };
                                let _ = shell_tx.send(UiCommand::ZoomLevel(level));
                            },
                            KeyCode::Enter => {
                                env.navigator.zoom_in(0);
                                let _ = shell_tx.send(UiCommand::ZoomLevel(2));
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
            thread::sleep(Duration::from_millis(16));
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
        // Headless mode — just run the brain for a bit
        println!("[Headless] No GUI. Running logic only.");
        thread::sleep(Duration::from_secs(2));
        println!("[Headless] Done.");
    }
}
