use tos_alpha2::brain::Brain;
use tos_alpha2::face::Face;
use tos_alpha2::platform::RemoteServer;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;36m[TOS BRAIN NODE: Component Under Test]\x1B[0m");
    
    // 1. Initialize Brain
    let brain = Brain::new()?;
    let ipc = brain.ipc.clone();
    let state = brain.state.clone();
    
    // 2. Initialize Face (for visualization)
    let mut face = Face::new(state.clone(), ipc.clone());

    // 3. Start Remote Server in background to receive stimulus
    let server = RemoteServer::new(ipc.clone());
    tokio::spawn(async move {
        if let Err(e) = server.run(7000).await {
            eprintln!("[BRAIN_NODE] Server failure: {}", e);
        }
    });

    println!("[BRAIN_NODE] Operational. Waiting for stimulus on port 7000...");

    // 4. Persistence Loop (Renders state updates)
    loop {
        // Clearing is disabled for demo visibility, but we re-render on a tick
        face.render();
        sleep(Duration::from_millis(500)).await;
        
        // Check if we should exit (e.g. via a specific IPC command we could add)
        // For now, keep it alive for the Stimulator
    }
}
