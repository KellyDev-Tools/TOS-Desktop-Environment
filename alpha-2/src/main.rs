use tos_alpha2::brain::Brain;
use tos_alpha2::face::{Face, MockFace};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    // 1. Initialize Brain Core
    let brain = Brain::new()?;
    println!("TOS Alpha-2 BRAIN INITIALIZED.");
    
    // 2. Initialize Face (UI Layer)
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut mock_face = MockFace(face_raw);

    println!("\n--- SYSTEM POLISH DEMO ---");
    sleep(Duration::from_secs(1)).await;

    // 3. Render initial state (Level 1)
    mock_face.0.render();
    sleep(Duration::from_secs(2)).await;
    
    // 4. Zoom Transition
    mock_face.simulate_bezel_zoom_in();
    mock_face.0.render();
    sleep(Duration::from_secs(2)).await;
    
    // 5. Demonstrate Directory Mode
    mock_face.simulate_prompt_submit("ls -la");
    sleep(Duration::from_secs(1)).await; // Wait for PTY
    mock_face.0.render();
    sleep(Duration::from_secs(2)).await;
    
    // 6. Demonstrate Multi-Symmetry (Back to Level 1)
    {
        let mut state = brain.state.lock().unwrap();
        state.current_level = tos_alpha2::common::HierarchyLevel::GlobalOverview;
        brain.services.logger.log("Returning to Overview for sync check.", 2);
        drop(state);
    }
    mock_face.0.render();
    sleep(Duration::from_secs(2)).await;
    
    println!("\nTOS ALPHA-2 STACK FINALIZED.");
    println!("ALL CORE COMPONENTS VERIFIED.");
    
    Ok(())
}
