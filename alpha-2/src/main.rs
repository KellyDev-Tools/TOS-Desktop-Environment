use tos_alpha2::brain::Brain;
use tos_alpha2::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    // 1. Initialize Brain (Logic, State, Shell)
    let brain = Brain::new()?;
    println!("TOS Alpha-2 Brain Initialized.");
    
    // 2. Initialize Face (UI Layer)
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut mock_face = MockFace(face_raw);

    println!("\n--- Phase 5 Demo: UI Transition & IPC ---");

    // 3. Render Level 1
    mock_face.0.render();
    
    // 4. Simulate Zoom In (L1 -> L2)
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    mock_face.simulate_bezel_zoom_in();
    mock_face.0.render();
    
    // 5. Submit a command via Face
    mock_face.simulate_prompt_submit("ls -la");
    
    // 6. Wait for PTY output and re-render
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    mock_face.0.render();
    
    println!("\nPhase 5 Demo Complete.");
    
    Ok(())
}
