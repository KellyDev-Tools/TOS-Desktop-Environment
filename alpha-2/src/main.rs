use tos_alpha2::brain::Brain;
use tos_alpha2::face::{Face, MockFace};
use tos_alpha2::platform::RemoteServer;
use std::time::Duration;
use tokio::time::sleep;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = env::args().collect();
    let is_self_test = args.iter().any(|arg| arg == "--self-test");

    // 1. Initialize Brain Core
    let brain = Brain::new()?;
    let ipc = brain.ipc.clone();
    let state = brain.state.clone();
    
    // 2. Initialize Face (UI Layer)
    let face_raw = Face::new(state.clone(), ipc.clone());
    let mut mock_face = MockFace(face_raw);

    if is_self_test {
        println!("\n--- SYSTEM SELF-TEST SEQUENCE ---");
        sleep(Duration::from_secs(1)).await;

        // Render initial state (Level 1)
        mock_face.0.render();
        sleep(Duration::from_secs(2)).await;
        
        // Zoom Transition
        mock_face.simulate_bezel_zoom_in();
        mock_face.0.render();
        sleep(Duration::from_secs(2)).await;
        
        // Demonstrate Directory Mode
        mock_face.simulate_prompt_submit("ls -la");
        sleep(Duration::from_secs(1)).await; // Wait for PTY
        mock_face.0.render();
        sleep(Duration::from_secs(2)).await;
        
        println!("\nSELF-TEST SEQUENCE COMPLETE.");
    } else {
        println!("TOS Alpha-2 BRAIN OPERATIONAL.");
        
        // Start IPC Server for Web UI (Port 7000)
        let server = RemoteServer::new(ipc.clone());
        tokio::spawn(async move {
            if let Err(e) = server.run(7000).await {
                eprintln!("[BRAIN] IPC Server failure: {}", e);
            }
        });

        println!("[BRAIN] Awaiting IPC stimulus on port 7000/7001...");

        // Main Loop: Periodic Rendering for Local Display (Terminal Dash)
        loop {
            // Check for mode change and force a re-render
            mock_face.0.render();
            sleep(Duration::from_millis(1000)).await;
        }
    }
    
    Ok(())
}
